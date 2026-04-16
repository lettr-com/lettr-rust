use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

fn webhook_json() -> serde_json::Value {
    serde_json::json!({
        "id": "webhook-abc123",
        "name": "Order Notifications",
        "url": "https://example.com/webhook",
        "enabled": true,
        "event_types": ["message.delivery", "message.bounce"],
        "auth_type": "basic",
        "has_auth_credentials": true,
        "last_successful_at": "2024-01-15T10:30:00+00:00",
        "last_failure_at": null,
        "last_status": "success"
    })
}

#[tokio::test]
async fn list_webhooks() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/webhooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Webhooks retrieved successfully.",
            "data": {
                "webhooks": [webhook_json()]
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let webhooks = client.webhooks.list().await.unwrap();

    assert_eq!(webhooks.len(), 1);
    assert_eq!(webhooks[0].id, "webhook-abc123");
    assert_eq!(webhooks[0].name, "Order Notifications");
    assert!(webhooks[0].enabled);
    assert_eq!(webhooks[0].auth_type, "basic");
    assert!(webhooks[0].has_auth_credentials);
    assert_eq!(webhooks[0].last_status.as_deref(), Some("success"));
    assert_eq!(webhooks[0].event_types.as_ref().unwrap().len(), 2);
}

#[tokio::test]
async fn get_webhook() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/webhooks/webhook-abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Webhook retrieved successfully.",
            "data": webhook_json()
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let webhook = client.webhooks.get("webhook-abc123").await.unwrap();

    assert_eq!(webhook.id, "webhook-abc123");
    assert_eq!(webhook.url, "https://example.com/webhook");
}

#[tokio::test]
async fn create_webhook() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/webhooks"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Webhook created successfully.",
            "data": webhook_json()
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let options = lettr::webhooks::CreateWebhookOptions::new(
        "Order Notifications",
        "https://example.com/webhook",
        "basic",
        "selected",
    )
    .with_events(vec!["message.delivery".into(), "message.bounce".into()])
    .with_basic_auth("user", "secret");

    let webhook = client.webhooks.create(options).await.unwrap();
    assert_eq!(webhook.id, "webhook-abc123");
    assert_eq!(webhook.name, "Order Notifications");
}

#[tokio::test]
async fn update_webhook() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/webhooks/webhook-abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Webhook updated successfully.",
            "data": {
                "id": "webhook-abc123",
                "name": "Updated Webhook",
                "url": "https://example.com/webhook",
                "enabled": false,
                "event_types": null,
                "auth_type": "none",
                "has_auth_credentials": false,
                "last_successful_at": null,
                "last_failure_at": null,
                "last_status": null
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let options = lettr::webhooks::UpdateWebhookOptions::new()
        .with_name("Updated Webhook")
        .with_active(false);

    let webhook = client.webhooks.update("webhook-abc123", options).await.unwrap();
    assert_eq!(webhook.name, "Updated Webhook");
    assert!(!webhook.enabled);
}

#[tokio::test]
async fn delete_webhook() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/webhooks/webhook-abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Webhook deleted successfully."
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    client.webhooks.delete("webhook-abc123").await.unwrap();
}

#[test]
fn create_webhook_serialization() {
    let options = lettr::webhooks::CreateWebhookOptions::new(
        "Test Webhook",
        "https://example.com/hook",
        "oauth2",
        "selected",
    )
    .with_events(vec!["delivery".into()])
    .with_oauth2("client-id", "client-secret", "https://auth.example.com/token");

    let json = serde_json::to_value(&options).unwrap();
    assert_eq!(json["name"], "Test Webhook");
    assert_eq!(json["url"], "https://example.com/hook");
    assert_eq!(json["auth_type"], "oauth2");
    assert_eq!(json["events_mode"], "selected");
    assert_eq!(json["events"][0], "delivery");
    assert_eq!(json["oauth_client_id"], "client-id");
    assert_eq!(json["oauth_client_secret"], "client-secret");
    assert_eq!(json["oauth_token_url"], "https://auth.example.com/token");
}

#[test]
fn update_webhook_serialization() {
    let options = lettr::webhooks::UpdateWebhookOptions::new()
        .with_name("New Name")
        .with_target("https://new.example.com/hook")
        .with_active(true);

    let json = serde_json::to_value(&options).unwrap();
    assert_eq!(json["name"], "New Name");
    assert_eq!(json["target"], "https://new.example.com/hook");
    assert_eq!(json["active"], true);
    // Fields not set should not be present
    assert!(json.get("auth_type").is_none());
    assert!(json.get("events").is_none());
}
