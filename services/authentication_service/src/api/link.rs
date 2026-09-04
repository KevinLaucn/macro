use crate::api::context::ApiContext;
#[cfg(feature = "full-saas")]
use axum::routing::delete;
use axum::{
    Router,
    routing::{get, post},
};
pub(in crate::api) mod create_in_progress_link;
#[cfg(feature = "full-saas")]
pub(in crate::api) mod github;
pub(in crate::api) mod gmail;
#[cfg(feature = "full-saas")]
pub(in crate::api) mod outlook;

/// Routes for authenticated account-link operations.
pub fn router() -> Router<ApiContext> {
    let router = Router::new()
        .route("/", post(create_in_progress_link::handler))
        .route("/gmail", post(gmail::init_gmail_link_handler))
        .route("/gmail/status", get(gmail::check_gmail_link_status_handler));

    #[cfg(feature = "full-saas")]
    let router = router
        .route("/github", post(github::init_github_link_handler))
        .route("/github", delete(github::delete_github_link_handler))
        .route(
            "/github/status",
            get(github::check_github_link_status_handler),
        )
        .route("/outlook", post(outlook::init_outlook_link_handler));

    router
}
