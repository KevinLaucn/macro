pub(crate) mod access;
#[cfg(feature = "calendar")]
pub(crate) mod calendar;
pub(crate) mod delete;
pub(crate) mod health_check;
pub(crate) mod list;
pub(crate) mod resync;

use crate::api::ApiContext;
use axum::Router;
use axum::routing::{delete, get, post};

pub fn router() -> Router<ApiContext> {
    let router = Router::new()
        .route("/", get(list::list_links_handler))
        .route("/health-check", post(health_check::health_check_handler))
        .route("/{link_id}", delete(delete::delete_link_handler))
        .route("/{link_id}/resync", post(resync::resync_link_handler));

    #[cfg(feature = "calendar")]
    {
        router.route(
            "/{link_id}/calendar",
            delete(calendar::disable_link_calendar_handler),
        )
    }

    #[cfg(not(feature = "calendar"))]
    {
        router
    }
}
