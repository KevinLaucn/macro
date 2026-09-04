use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use connection::domain::{
    models::{ConnectionError, InvalidationEvent},
    ports::ConnectionService,
};
use documents_hex::{
    domain::{
        models::{
            CloudFrontConfig, CreateDocumentRepoArgs, DocumentError, ImportEmailAttachmentRepoArgs,
        },
        ports::create::DocumentCreationService,
        ports::{DocumentService, TaskPropertiesPort},
        response::CreateDocumentResponse,
        service::DocumentServiceImpl,
    },
    outbound::{pg_document_repo::PgDocumentRepo, s3_upload_url::S3UploadUrlAdapter},
};
use entity_access_management::{
    domain::service::EntityAccessManagementServiceImpl,
    outbound::PgRepository as AccessManagementRepo,
};
use foreign_entity::{
    domain::service::ForeignEntityServiceImpl,
    outbound::pg_foreign_entity_repo::PgForeignEntityRepo,
};
use macro_entrypoint::MacroEntrypoint;
use macro_env_var::env_var;
use macro_event_broker::NoopMacroEventBroker;
use macro_service_urls::SyncServiceUrl;
use macro_tower_layers::MacroRequestIdAndTracingLayer;
use model::document::response::CreateDocumentRequest;
use model::document::{FileType, FileTypeExt};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

use crate::config::{Config, Environment};

const INTERNAL_AUTH_HEADER: &str = "x-document-storage-service-auth-key";
const INTERNAL_USER_ID_HEADER: &str = "x-document-storage-service-user-id";

env_var! {
    #[derive(Clone)]
    pub struct DocumentStorageServiceAuthKey;
}

type LeanEntityAccessManagementService = EntityAccessManagementServiceImpl<AccessManagementRepo>;
type LeanForeignEntityService = ForeignEntityServiceImpl<PgForeignEntityRepo>;
type LeanDocumentService = DocumentServiceImpl<
    PgDocumentRepo,
    S3UploadUrlAdapter,
    NoopTaskProperties,
    NoopConnectionService,
    LeanEntityAccessManagementService,
    LeanForeignEntityService,
    NoopMacroEventBroker,
>;

#[derive(Clone)]
struct LeanApiState {
    config: Arc<Config>,
    auth_key: DocumentStorageServiceAuthKey,
    document_service: Arc<LeanDocumentService>,
}

#[derive(Clone)]
struct NoopTaskProperties;

impl TaskPropertiesPort for NoopTaskProperties {
    async fn attach_task_properties(&self, _entity_ids: Vec<String>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn update_task_status(&self, _task_id: &str, _status: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn set_entity_property(
        &self,
        _user_id: &str,
        _entity_id: &str,
        _property_definition_id: uuid::Uuid,
        _value: Option<models_properties::api::requests::SetPropertyValue>,
        _attribution: &activity::Attribution,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn copy_task_properties(
        &self,
        _from_task_id: &str,
        _to_task_id: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
struct NoopConnectionService;

impl ConnectionService for NoopConnectionService {
    async fn send_invalidation_event<'a, T: std::fmt::Debug + serde::Serialize + Send>(
        &self,
        _invalidation_event: InvalidationEvent<'a, T>,
    ) -> Result<(), ConnectionError> {
        Ok(())
    }

    async fn send_channel_message<'a>(
        &self,
        _users: &[macro_user_id::user_id::MacroUserIdStr<'a>],
        _message_type: &str,
        _message: serde_json::Value,
    ) -> Result<(), ConnectionError> {
        Ok(())
    }
}

pub(crate) async fn run() -> anyhow::Result<()> {
    let entrypoint = MacroEntrypoint::default().init();
    let result = run_inner().await;
    entrypoint.shutdown();
    result
}

async fn run_inner() -> anyhow::Result<()> {
    let env = Environment::new_or_prod();
    let aws_config = macro_aws_config::get_macro_aws_config().await;
    let secretsmanager_client = secretsmanager_client::SecretsManager::new(
        aws_sdk_secretsmanager::Client::new(&aws_config),
    );

    let config = Config::from_env()
        .context("expected to be able to generate config")?
        .resolve_remote_secrets(env, &secretsmanager_client)
        .await
        .context("expected to be able to resolve config secrets")?;

    let (min_connections, max_connections): (u32, u32) = match config.environment {
        Environment::Production => (50, 150),
        Environment::Develop => (25, 100),
        Environment::Local => (15, 50),
    };

    let db = PgPoolOptions::new()
        .min_connections(min_connections)
        .max_connections(max_connections)
        .connect(&config.database_url)
        .await
        .context("could not connect to db")?;

    let sync_service_client = sync_service_client::SyncServiceClient::new(
        config.sync_service_auth_key.as_ref().to_string(),
        SyncServiceUrl::new()?.to_string(),
    );
    let cloudfront_config = CloudFrontConfig {
        distribution_url: config
            .document_storage_service_cloudfront_distribution_url
            .as_ref()
            .to_string(),
        signer_public_key_id: config
            .document_storage_service_cloudfront_signer_public_key_id
            .as_ref()
            .to_string(),
        signer_private_key: config
            .document_storage_service_cloudfront_signer_private_key
            .as_ref()
            .to_string(),
        presigned_url_expiry_seconds: config.document_storage_service_presigned_url_expiry_seconds,
        browser_cache_expiry_seconds: config
            .document_storage_service_presigned_url_browser_cache_expiry_seconds,
    };
    let document_service = Arc::new(DocumentServiceImpl::new(
        PgDocumentRepo::new(db.clone()),
        cloudfront_config,
        sync_service_client,
        S3UploadUrlAdapter::new(
            macro_aws_config::s3_client().await,
            config.document_storage_bucket.as_ref(),
            config.docx_document_upload_bucket.as_ref(),
        ),
        NoopTaskProperties,
        NoopConnectionService,
        EntityAccessManagementServiceImpl::new(AccessManagementRepo::new(db.clone())),
        ForeignEntityServiceImpl::new(PgForeignEntityRepo::new(db.clone())),
        NoopMacroEventBroker,
    ));

    let state = LeanApiState {
        auth_key: DocumentStorageServiceAuthKey::new()?,
        config: Arc::new(config),
        document_service,
    };

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/internal/health", get(internal_health_handler))
        .route(
            "/internal/documents",
            post(create_document_internal_handler),
        )
        .route(
            "/{version}/internal/documents",
            post(create_document_internal_handler),
        )
        .with_state(state.clone())
        .layer(
            ServiceBuilder::new()
                .layer(MacroRequestIdAndTracingLayer::new(Duration::from_millis(200)).into_inner())
                .layer(macro_cors::cors_layer())
                .layer(CompressionLayer::new().gzip(true)),
        );

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", state.config.port))
        .await
        .context("could not bind document storage service listener")?;
    tracing::info!(
        "lean document storage service is up and running with environment {:?} on port {}",
        &state.config.environment,
        &state.config.port
    );
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(macro_entrypoint::shutdown_signal())
        .await
        .context("error starting service")
}

async fn health_handler() -> &'static str {
    "healthy"
}

async fn internal_health_handler(
    State(state): State<LeanApiState>,
    headers: HeaderMap,
) -> Result<&'static str, LeanApiError> {
    ensure_internal_auth(&state, &headers)?;
    Ok("healthy")
}

async fn create_document_internal_handler(
    State(state): State<LeanApiState>,
    headers: HeaderMap,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<CreateDocumentResponse>, LeanApiError> {
    ensure_internal_auth(&state, &headers)?;
    let user_id = headers
        .get(INTERNAL_USER_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or(LeanApiError::MissingUserId)?
        .to_string();

    let user_provided_file_type: Option<FileType> = req
        .file_type
        .as_deref()
        .and_then(|file_type| file_type.parse().ok());
    let (document_name, file_type) = match user_provided_file_type {
        Some(file_type) => {
            let document_name = FileType::clean_document_name(&req.document_name);
            (document_name.unwrap_or(req.document_name), Some(file_type))
        }
        None => match FileType::split_suffix_match(req.document_name.as_str()) {
            Some((file_name, extension)) => (file_name.to_string(), extension.parse().ok()),
            None => (req.document_name, None),
        },
    };

    if let (Some(file_type), Some(user_mime_type)) = (file_type, &req.mime_type)
        && *user_mime_type != file_type.mime_type()
    {
        tracing::warn!(
            file_type=?file_type,
            mime_type=?user_mime_type,
            "provided mime type does not match file type"
        );
    }

    let user_id = macro_user_id::user_id::MacroUserIdStr::try_from(user_id)
        .map_err(|error| LeanApiError::BadUserId(error.to_string()))?;
    let create = CreateDocumentRepoArgs {
        id: req.id,
        sha: req.sha,
        document_name,
        user_id: user_id.clone(),
        file_type,
        project_id: req.project_id,
        team_id: req.team_id,
        created_at: req.created_at,
        sub_type: req
            .is_task
            .then_some(document_sub_type::DocumentSubType::Task),
        skip_history: req.skip_history,
        attribution: None,
    };

    let data = if let Some(email_attachment_id) = req.email_attachment_id {
        state
            .document_service
            .import_email_attachment(
                user_id,
                ImportEmailAttachmentRepoArgs {
                    email_attachment_id,
                    create,
                },
            )
            .await?
    } else {
        state
            .document_service
            .create_document(user_id, create, req.job_id)
            .await?
    };

    Ok(Json(CreateDocumentResponse { error: false, data }))
}

fn ensure_internal_auth(state: &LeanApiState, headers: &HeaderMap) -> Result<(), LeanApiError> {
    let Some(auth) = headers
        .get(INTERNAL_AUTH_HEADER)
        .and_then(|value| value.to_str().ok())
    else {
        return Err(LeanApiError::Unauthorized);
    };
    if auth != state.auth_key.as_ref() {
        return Err(LeanApiError::Unauthorized);
    }
    Ok(())
}

enum LeanApiError {
    BadUserId(String),
    Document(DocumentError),
    MissingUserId,
    Unauthorized,
}

impl From<DocumentError> for LeanApiError {
    fn from(error: DocumentError) -> Self {
        Self::Document(error)
    }
}

impl IntoResponse for LeanApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::BadUserId(error) => (
                StatusCode::BAD_REQUEST,
                format!("invalid internal user id: {error}"),
            )
                .into_response(),
            Self::Document(DocumentError::NotFound(error)) => {
                (StatusCode::NOT_FOUND, error).into_response()
            }
            Self::Document(DocumentError::Unauthorized) | Self::Unauthorized => {
                StatusCode::UNAUTHORIZED.into_response()
            }
            Self::Document(DocumentError::Gone) => StatusCode::GONE.into_response(),
            Self::Document(DocumentError::Conflict(error)) => {
                (StatusCode::CONFLICT, error).into_response()
            }
            Self::Document(DocumentError::BadRequest(error)) => {
                (StatusCode::BAD_REQUEST, error).into_response()
            }
            Self::Document(DocumentError::NameTooLong { max }) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("document name exceeds max length {max}"),
            )
                .into_response(),
            Self::Document(DocumentError::Internal(error)) => {
                tracing::error!(error=?error, "internal document storage service error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            Self::MissingUserId => (
                StatusCode::BAD_REQUEST,
                format!("missing {INTERNAL_USER_ID_HEADER} header"),
            )
                .into_response(),
        }
    }
}
