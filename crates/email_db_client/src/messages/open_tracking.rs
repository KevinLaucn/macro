//! Read-receipt (open) tracking for sent messages.
//!
//! At send time a unique token is stored on the outgoing message and embedded
//! in a tracking pixel URL inside the message HTML. Opens are recorded by the
//! email service's public pixel endpoint, keyed by that token.

use sqlx::PgPool;
use sqlx::types::Uuid;

/// Assigns the open-tracking token embedded in an outgoing message's tracking
/// pixel. Errors if the message doesn't exist for `(message_id, link_id)`, so
/// callers never inject a pixel whose token wasn't persisted.
#[tracing::instrument(skip(pool), err)]
pub async fn set_message_open_tracking_token(
    pool: &PgPool,
    message_id: Uuid,
    link_id: Uuid,
    token: Uuid,
) -> anyhow::Result<()> {
    let result = sqlx::query(
        r#"
        UPDATE email_messages
        SET open_tracking_token = $3
        WHERE id = $1 AND link_id = $2
        "#,
    )
    .bind(message_id)
    .bind(link_id)
    .bind(token)
    .execute(pool)
    .await?;

    if result.rows_affected() != 1 {
        anyhow::bail!(
            "expected to set open tracking token on exactly one message, but {} rows matched (message_id={}, link_id={})",
            result.rows_affected(),
            message_id,
            link_id
        );
    }

    Ok(())
}

/// Records an open for a sent message identified by `token`. Returns the
/// message/link/thread IDs and updated open count when a row matched.
#[derive(Debug, Clone)]
pub struct RecordedOpen {
    pub message_id: Uuid,
    pub link_id: Uuid,
    pub thread_db_id: Uuid,
    pub open_count: i32,
}

#[tracing::instrument(skip(pool, token), err)]
pub async fn record_message_open(
    pool: &PgPool,
    token: Uuid,
) -> anyhow::Result<Option<RecordedOpen>> {
    let row = sqlx::query_as::<_, (Uuid, Uuid, Uuid, i32)>(
        r#"
        UPDATE email_messages
        SET first_opened_at = COALESCE(first_opened_at, NOW()),
            last_opened_at = NOW(),
            open_count = open_count + 1
        WHERE open_tracking_token = $1 AND is_sent = true
        RETURNING id, link_id, thread_id, open_count
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(
        |(message_id, link_id, thread_db_id, open_count)| RecordedOpen {
            message_id,
            link_id,
            thread_db_id,
            open_count,
        },
    ))
}
