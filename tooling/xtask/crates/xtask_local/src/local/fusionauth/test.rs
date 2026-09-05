use super::*;
use serde_json::json;

#[test]
fn reconcile_requests_are_scoped_to_local_admin_and_google_config() {
    let doc = json!({
        "requests": [
            {
                "method": "PATCH",
                "url": "/api/tenant/11111111-1111-4111-8111-111111111111",
                "body": { "tenant": {} }
            },
            {
                "method": "POST",
                "url": "/api/user/registration",
                "body": { "user": { "email": "owner@example.com", "password": "secret" } }
            },
            {
                "method": "POST",
                "url": "/api/lambda/66666666-6666-4666-8666-666666666666",
                "body": { "lambda": {} }
            },
            {
                "method": "POST",
                "url": "/api/identity-provider/44444444-4444-4444-8444-444444444444",
                "body": { "identityProvider": { "name": "google" } }
            },
            {
                "method": "POST",
                "url": "/api/identity-provider/55555555-5555-4555-8555-555555555555",
                "body": { "identityProvider": { "name": "google_gmail" } }
            },
            {
                "method": "POST",
                "url": "/api/webhook",
                "body": { "webhook": {} }
            }
        ]
    });

    let urls = reconcile_requests(&doc)
        .into_iter()
        .map(|request| {
            request
                .get("url")
                .and_then(serde_json::Value::as_str)
                .unwrap()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        urls,
        vec![
            "/api/user/registration",
            "/api/lambda/66666666-6666-4666-8666-666666666666",
            "/api/identity-provider/44444444-4444-4444-8444-444444444444",
            "/api/identity-provider/55555555-5555-4555-8555-555555555555",
        ]
    );
}
