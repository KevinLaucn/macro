/// AI-backed summarizer implementing [`CallSummarizer`](crate::domain::ports::CallSummarizer).
pub mod ai_call_summarizer;
/// LiveKit RTC client adapter implementing [`CallRtcClient`](crate::domain::ports::CallRtcClient).
#[cfg(feature = "livekit")]
pub mod livekit_rtc_client;
/// Postgres-backed repository implementing [`CallRepository`](crate::domain::ports::CallRepository).
pub mod pg_call_repo;
/// Postgres-backed repository implementing [`VoiceRepository`](crate::domain::ports::VoiceRepository).
#[cfg(feature = "voice")]
pub mod pg_voice_repo;
/// S3-backed recording storage implementing [`RecordingStorage`](crate::domain::ports::RecordingStorage).
pub mod s3_recording_storage;

/// Stub RTC client for deployments where real-time calling is disabled.
#[derive(Debug, Default, Clone, Copy)]
pub struct DisabledCallRtcClient;

impl DisabledCallRtcClient {
    /// Create a new disabled RTC client.
    pub fn new() -> Self {
        Self
    }
}

impl crate::domain::ports::CallRtcClient for DisabledCallRtcClient {
    async fn create_room(&self, _room_name: &str) -> anyhow::Result<()> {
        anyhow::bail!("Calls feature is disabled in this deployment")
    }

    async fn delete_room(&self, _room_name: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn generate_token<'a>(
        &self,
        _room_name: &str,
        _participant_identity: macro_user_id::user_id::MacroUserIdStr<'a>,
    ) -> anyhow::Result<String> {
        anyhow::bail!("Calls feature is disabled in this deployment")
    }

    async fn build_voip_push_payloads<'a>(
        &self,
        _request: crate::domain::models::VoipPushPayloadRequest<'a>,
    ) -> Vec<(
        macro_user_id::user_id::MacroUserIdStr<'static>,
        notification::domain::models::apple::VoipPushPayload,
    )> {
        Vec::new()
    }

    async fn remove_participant<'a>(
        &self,
        _room_name: &str,
        _participant_identity: macro_user_id::user_id::MacroUserIdStr<'a>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn start_room_composite_egress(
        &self,
        _room_name: &str,
        _s3_config: &crate::domain::models::EgressS3Config,
    ) -> anyhow::Result<String> {
        anyhow::bail!("Calls feature is disabled in this deployment")
    }

    async fn stop_egress(&self, _egress_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn receive_webhook(
        &self,
        _body: &str,
        _auth_token: &str,
    ) -> Result<crate::domain::models::CallWebhookEvent, crate::domain::models::CallError> {
        Err(crate::domain::models::CallError::Auth)
    }

    fn verify_access_token(
        &self,
        _token: &str,
    ) -> anyhow::Result<crate::domain::models::VerifiedRingToken> {
        anyhow::bail!("Calls feature is disabled in this deployment")
    }

    async fn dispatch_transcription_agent(&self, _room_name: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
