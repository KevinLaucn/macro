use crate::api::ApiContext;
#[cfg(feature = "full-processing")]
use axum::routing::delete;
use axum::{Router, routing::post};

pub(in crate::api) mod backfill;
#[cfg(feature = "full-processing")]
pub(in crate::api) mod delete_document;
#[cfg(feature = "full-processing")]
pub(in crate::api) mod extract_sync;

pub fn router() -> Router<ApiContext> {
    let router = Router::new().nest("/backfill", backfill::router());

    #[cfg(feature = "full-processing")]
    let router = router
        .route("/delete/{document_id}", delete(delete_document::handler))
        .route("/extract_sync", post(extract_sync::handler));

    router
}
