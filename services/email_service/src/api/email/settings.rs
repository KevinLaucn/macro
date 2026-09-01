use crate::api::context::ApiContext;
use crate::api::email::settings::patch::patch_settings_handler;
use axum::Router;
use axum::routing::{get, patch};

pub(crate) mod patch;
pub(crate) mod read_receipts;

pub fn router(state: ApiContext) -> Router<ApiContext> {
    Router::new()
        .route("/", patch(patch_settings_handler))
        .route(
            "/read-receipts",
            get(read_receipts::get_handler).patch(read_receipts::patch_handler),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::api::middleware::link::attach_link_context,
        ))
}
