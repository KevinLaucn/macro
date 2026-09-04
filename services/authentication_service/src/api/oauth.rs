use axum::{Router, routing::get};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;

use crate::api::ApiContext;

#[cfg(feature = "full-saas")]
use super::middleware;

// needs to be public in api crate for swagger
pub(in crate::api) mod oauth_redirect;
#[cfg(feature = "full-saas")]
pub(in crate::api) mod passwordless_callback;

pub fn router(_state: ApiContext) -> Router<ApiContext> {
    let router = Router::new().route(
        "/redirect",
        get(oauth_redirect::handler).layer(ServiceBuilder::new().layer(CookieManagerLayer::new())),
    );

    #[cfg(feature = "full-saas")]
    let router = router.route(
        "/passwordless/{code}",
        get(passwordless_callback::handler).layer(
            ServiceBuilder::new()
                .layer(CookieManagerLayer::new())
                .layer(axum::middleware::from_fn_with_state(
                    _state.clone(),
                    middleware::rate_limit::login_code::handler,
                )),
        ),
    );

    router
}
