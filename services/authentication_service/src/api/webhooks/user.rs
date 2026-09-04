use crate::api::context::ApiContext;
use axum::{Router, routing::post};

mod create_user_webhook;
#[cfg(feature = "full-saas")]
mod delete_user_webhook;
mod populate_jwt;
#[cfg(feature = "full-saas")]
mod stripe_webhook;
mod update_name;

pub fn router() -> Router<ApiContext> {
    let router = Router::new()
        .route("/", post(create_user_webhook::handler))
        .route("/jwt", post(populate_jwt::handler))
        .route("/name", post(update_name::handler));

    #[cfg(feature = "full-saas")]
    let router = router
        .route("/delete", post(delete_user_webhook::handler))
        .route("/stripe", post(stripe_webhook::handler));

    router
}
