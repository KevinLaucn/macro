use std::sync::Arc;

#[cfg(feature = "full-saas")]
use analytics_client::AnalyticsClient;
use axum::extract::FromRef;
#[cfg(feature = "full-saas")]
use channels::{
    domain::{
        service::ChannelServiceImpl,
        side_effects::{ChannelSideEffectService, SpawnedChannelEventDispatcher},
    },
    outbound::{
        connection_gateway_realtime::ConnectionGatewayChannelRealtimePublisher,
        contacts_dispatcher::ContactsChannelDispatcher,
        notification_sender::NotificationChannelSender,
        pg_channel_reference_share_permissions::PgChannelReferenceSharePermissions,
        pg_channels_repo::PgChannelsRepo, pg_side_effect_context::PgChannelSideEffectContext,
    },
};
#[cfg(feature = "full-saas")]
use contacts::{domain::service::SqsContactsIngress, outbound::ingress::SqsContactsQueue};
use entity_access::domain::service::EntityAccessServiceImpl;
use entity_access::outbound::PgAccessRepository;
#[cfg(feature = "full-saas")]
use foreign_entity::domain::service::ForeignEntityServiceImpl;
#[cfg(feature = "full-saas")]
use foreign_entity::outbound::pg_foreign_entity_repo::PgForeignEntityRepo;
#[cfg(feature = "full-saas")]
use github::domain::service::GithubLinkServiceImpl;
#[cfg(feature = "full-saas")]
use github::outbound::github_auth_client::GithubAuthImpl;
#[cfg(feature = "full-saas")]
use github::outbound::github_oauth_client::GithubOauthImpl;
#[cfg(feature = "full-saas")]
use github::outbound::pg_github_repo::PgGithubRepo;
#[cfg(feature = "full-saas")]
use loops_client::LoopsClient;
use macro_auth::{InternalApiKey, middleware::decode_jwt::JwtValidationArgs};
use macro_authorization::{
    MacroAuthJwtValidator, MacroAuthorizationServiceImpl, MacroAuthorizationState,
};
use macro_cache_client::MacroCache;
use macro_env::Environment;
use macro_env_var::env_var;
#[cfg(feature = "full-saas")]
use macro_event_broker::{KafkaEventPublisher, MacroEventBrokerService};
#[cfg(feature = "full-saas")]
use native_app_service::{domain::service::NativeAppServiceImpl, outbound::DefaultBundleFetcher};
#[cfg(feature = "full-saas")]
use notification::outbound::queue::SqsQueue;
#[cfg(feature = "full-saas")]
use notification::{
    domain::service::SqsNotificationIngress, outbound::rate_limit::RedisRateLimitAdapter,
};
#[cfg(feature = "full-saas")]
use rate_limit::domain::service::RateLimitServiceImpl;
#[cfg(feature = "full-saas")]
use referral::{
    domain::service::ReferralServiceImpl,
    outbound::{pg_referral_repo::PgReferralRepo, stripe_discount_client::StripeDiscountClient},
};
use remote_env_var::LocalOrRemoteSecret;
#[cfg(feature = "full-saas")]
use roles_and_permissions::{
    domain::service::UserRolesAndPermissionsServiceImpl, outbound::pgpool::MacroDB,
};
use sqlx::PgPool;
#[cfg(feature = "full-saas")]
use tokio_util::task::TaskTracker;

use crate::microsoft_token_cipher::MicrosoftTokenCipher;
use authentication_service::service::signup_policy::SignupPolicy;
#[cfg(feature = "full-saas")]
use cursor_api_key::cipher::CursorApiKeyCipher;

#[cfg(feature = "full-saas")]
pub(crate) type NotificationIngressType = SqsNotificationIngress<SqsQueue>;
#[cfg(feature = "full-saas")]
pub(crate) type AuthenticationEventBroker =
    MacroEventBrokerService<KafkaEventPublisher, TaskTracker>;

#[cfg(feature = "full-saas")]
pub(crate) type ChannelServiceType = ChannelServiceImpl<
    PgChannelsRepo,
    SpawnedChannelEventDispatcher<
        ChannelSideEffectService<
            PgChannelSideEffectContext,
            ConnectionGatewayChannelRealtimePublisher,
            NotificationChannelSender<NotificationIngressType>,
            ContactsChannelDispatcher<SqsContactsIngress<SqsContactsQueue>>,
            AuthenticationEventBroker,
        >,
    >,
    PgChannelReferenceSharePermissions<EntityAccessServiceType>,
>;

#[cfg(feature = "full-saas")]
pub(crate) type TeamsServiceType = teams::domain::team_service::TeamServiceImpl<
    teams::outbound::team_repo::TeamRepositoryImpl,
    teams::outbound::customer_repo::CustomerRepositoryImpl,
    ChannelServiceType,
    UserRolesAndPermissionsServiceImpl<MacroDB, MacroDB>,
    NotificationIngressType,
    teams::outbound::crm_enqueuer::SqsCrmEnqueuer,
    teams::outbound::team_crm_settings_repo::TeamCrmSettingsRepositoryImpl,
    teams::outbound::team_analytics::AnalyticsClientTeamAnalytics,
    teams::outbound::contacts_enqueuer::ContactsIngressEnqueuer<
        SqsContactsIngress<SqsContactsQueue>,
    >,
    AuthenticationEventBroker,
>;

#[cfg(feature = "full-saas")]
pub(crate) type RateLimiter = RateLimitServiceImpl<RedisRateLimitAdapter<redis::Client>>;

#[cfg(feature = "full-saas")]
pub(crate) type ReferralServiceType = ReferralServiceImpl<
    PgReferralRepo,
    StripeDiscountClient,
    Arc<SqsNotificationIngress<SqsQueue>>,
>;

#[cfg(feature = "full-saas")]
pub(crate) type GithubLinkServiceType = GithubLinkServiceImpl<
    PgGithubRepo,
    GithubOauthImpl,
    GithubAuthImpl,
    ForeignEntityServiceImpl<PgForeignEntityRepo>,
>;

pub(crate) type EntityAccessServiceType = EntityAccessServiceImpl<PgAccessRepository>;

#[cfg(feature = "full-saas")]
pub(crate) type FavoritesServiceType = favorites::domain::service::FavoritesServiceImpl<
    favorites::outbound::pg_favorites_repo::PgFavoritesRepo,
>;

pub(crate) type AuthorizationService = MacroAuthorizationServiceImpl<MacroAuthJwtValidator>;

#[derive(Clone, FromRef)]
pub(crate) struct ApiContext {
    pub db: PgPool,
    #[cfg(feature = "full-saas")]
    pub github_link_service: Arc<GithubLinkServiceType>,
    pub auth_client: Arc<fusionauth::FusionAuthClient>,
    pub microsoft_token_cipher: Option<Arc<dyn MicrosoftTokenCipher>>,
    /// Encrypts users' Cursor API keys.
    #[cfg(feature = "full-saas")]
    pub cursor_api_key_cipher: Arc<dyn CursorApiKeyCipher>,
    pub macro_cache_client: Arc<MacroCache>,
    #[cfg(feature = "full-saas")]
    pub stripe_client: Arc<stripe::Client>,
    #[cfg(feature = "full-saas")]
    pub document_storage_service_client:
        Arc<document_storage_service_client::DocumentStorageServiceClient>,
    pub email_service_client: Arc<email::outbound::EmailServiceHttpClient>,
    #[cfg(feature = "full-saas")]
    pub ses_client: Arc<ses_client::Ses>,
    #[cfg(feature = "full-saas")]
    pub notification_ingress_service: Arc<NotificationIngressType>,
    pub sqs_client: Arc<sqs_client::SQS>,
    pub environment: Environment,
    pub signup_policy: Arc<SignupPolicy>,
    pub jwt_args: JwtValidationArgs,
    pub authorization_state: MacroAuthorizationState<AuthorizationService>,
    pub token_context: MacroApiTokenContext,
    pub internal_api_key: InternalApiKey,
    #[cfg(feature = "full-saas")]
    pub stripe_webhook_secret: LocalOrRemoteSecret<StripeWebhookSecretKey>,
    #[cfg(feature = "full-saas")]
    pub user_roles_and_permissions_service:
        Arc<UserRolesAndPermissionsServiceImpl<MacroDB, MacroDB>>, // Note: since FromRef doesn't support generics we have to specify the concrete types here
    #[cfg(feature = "full-saas")]
    pub teams_service: Arc<TeamsServiceType>,
    #[cfg(feature = "full-saas")]
    pub channel_service: Arc<ChannelServiceType>,
    #[cfg(feature = "full-saas")]
    pub favorites_service: Arc<FavoritesServiceType>,
    pub entity_access_service: Arc<EntityAccessServiceType>,
    #[cfg(feature = "full-saas")]
    pub native_app_service: Arc<NativeAppServiceImpl<DefaultBundleFetcher>>,
    #[cfg(feature = "full-saas")]
    pub analytics_client: Arc<AnalyticsClient>,
    #[cfg(feature = "full-saas")]
    pub loops_client: Arc<LoopsClient>,
    #[cfg(feature = "full-saas")]
    pub referral_service: Arc<ReferralServiceType>,
    #[cfg(feature = "full-saas")]
    pub rate_limit_service: RateLimiter,
    /// The stripe price id
    #[cfg(feature = "full-saas")]
    pub stripe_price_id: String,
    /// Whether Gmail link consent requests the Google Calendar scope.
    pub calendar_scope_enabled: bool,
}

env_var! {
    #[derive(Clone)]
    pub struct StripeWebhookSecretKey;
}

env_var! {
    #[derive(Clone)]
    pub struct MacroApiTokenIssuer;
}
env_var! {
    #[derive(Clone)]
    pub struct MacroApiTokenPrivateSecretKey;
}

env_var! {
    #[derive(Clone)]
    pub struct MacroApiTokenExpirySeconds;
}

#[derive(Clone)]
pub struct MacroApiTokenContext {
    /// The issuer of the macro-api-token
    pub issuer: MacroApiTokenIssuer,
    /// The macro api token private key used to sign macro-api tokens
    pub macro_api_token_private_key: LocalOrRemoteSecret<MacroApiTokenPrivateSecretKey>,
    /// The token expiry duration in seconds
    pub expiry_seconds: usize,
}

#[derive(Clone)]
pub struct TokenContext {
    /// The access token
    pub access_token: String,
    /// The refresh token
    pub refresh_token: String,
}
