//! Gmail mailbox-subscription capability implementation.

use chrono::{TimeZone, Utc};

use crate::domain::models::{AccessToken, EmailApiError, ProviderSubscription, SyncCursor};
use crate::domain::ports::MailboxSubscriptionClient;

use super::{GmailApiClientRepository, is_watch_conflict, map_gmail_error, map_watch_error};

impl MailboxSubscriptionClient for GmailApiClientRepository {
    async fn subscribe(
        &self,
        access_token: &AccessToken,
    ) -> Result<ProviderSubscription, EmailApiError> {
        let token = access_token.expose_secret();
        let (history_id, expiration_str) = match self.client.register_watch(token).await {
            Ok(watch) => (watch.history_id, Some(watch.expiration)),
            Err(error) if is_watch_conflict(&error) => {
                tracing::warn!(
                    error = %error,
                    "Gmail watch conflict; stopping the existing watch and retrying once"
                );
                self.client
                    .stop_watch(token)
                    .await
                    .map_err(map_gmail_error)?;
                let watch = self
                    .client
                    .register_watch(token)
                    .await
                    .map_err(map_watch_error)?;
                (watch.history_id, Some(watch.expiration))
            }
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "Gmail register_watch failed; falling back to profile historyId so inbox can still be connected"
                );
                let profile = self
                    .client
                    .get_profile(token)
                    .await
                    .map_err(map_gmail_error)?;
                (profile.history_id, None)
            }
        };

        let expires_at = if let Some(exp) = expiration_str {
            let expiration = exp
                .parse::<i64>()
                .map_err(|error| EmailApiError::Permanent {
                    message: format!("Gmail watch returned an invalid expiration: {error}"),
                })?;
            Utc.timestamp_millis_opt(expiration)
                .single()
                .ok_or_else(|| EmailApiError::Permanent {
                    message: "Gmail watch returned an out-of-range expiration".to_string(),
                })?
        } else {
            Utc::now() + chrono::Duration::days(7)
        };

        Ok(ProviderSubscription::new(
            SyncCursor::gmail(history_id),
            expires_at,
        ))
    }

    async fn unsubscribe(&self, access_token: &AccessToken) -> Result<(), EmailApiError> {
        self.client
            .stop_watch(access_token.expose_secret())
            .await
            .map_err(map_gmail_error)
    }
}
