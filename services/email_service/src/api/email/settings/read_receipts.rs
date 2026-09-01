use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use models_email::service::link::Link;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::api::context::ApiContext;

#[derive(Debug, Clone, Serialize)]
pub struct ReadReceiptsResponse {
    pub read_receipts_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateReadReceiptsRequest {
    pub read_receipts_enabled: bool,
}

#[derive(Debug, Error)]
pub enum ReadReceiptsError {
    #[error("Failed to read or update read receipt settings")]
    Database(#[from] anyhow::Error),
}

impl IntoResponse for ReadReceiptsError {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self, "read receipt settings error");
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

#[tracing::instrument(skip(ctx, link))]
pub async fn get_handler(
    State(ctx): State<ApiContext>,
    link: Extension<Link>,
) -> Result<Json<ReadReceiptsResponse>, ReadReceiptsError> {
    let enabled = email_db_client::settings::fetch_read_receipts_enabled(&ctx.db, link.id).await?;
    Ok(Json(ReadReceiptsResponse {
        read_receipts_enabled: enabled,
    }))
}

#[tracing::instrument(skip(ctx, link, request))]
pub async fn patch_handler(
    State(ctx): State<ApiContext>,
    link: Extension<Link>,
    Json(request): Json<UpdateReadReceiptsRequest>,
) -> Result<Json<ReadReceiptsResponse>, ReadReceiptsError> {
    let enabled = email_db_client::settings::set_read_receipts_enabled(
        &ctx.db,
        link.id,
        request.read_receipts_enabled,
    )
    .await?;

    Ok(Json(ReadReceiptsResponse {
        read_receipts_enabled: enabled,
    }))
}
