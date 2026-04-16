use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

#[tokio::test]
async fn health_check() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Health check passed.",
            "data": {
                "status": "ok",
                "timestamp": "2024-01-15T10:00:00Z"
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let response = client.health().await.unwrap();

    assert_eq!(response.message, "Health check passed.");
    assert_eq!(response.data.status, "ok");
    assert_eq!(response.data.timestamp, "2024-01-15T10:00:00Z");
}

#[tokio::test]
async fn auth_check() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/auth/check"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "API key is valid.",
            "data": {
                "team_id": 42,
                "timestamp": "2024-01-15T10:00:00Z"
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let response = client.auth_check().await.unwrap();

    assert_eq!(response.message, "API key is valid.");
    assert_eq!(response.data.team_id, 42);
}

#[tokio::test]
async fn api_error_handling() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "message": "The sender domain could not be determined.",
            "error_code": "invalid_domain"
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let email = lettr::CreateEmailOptions::new(
        "bad@example.com",
        ["user@example.com"],
        "Test",
    )
    .with_html("<p>Test</p>");

    let err = client.emails.send(email).await.unwrap_err();
    match err {
        lettr::Error::Api(api_err) => {
            assert_eq!(api_err.message, "The sender domain could not be determined.");
            assert_eq!(api_err.error_code.as_deref(), Some("invalid_domain"));
        }
        other => panic!("Expected Api error, got: {other:?}"),
    }
}

#[tokio::test]
async fn validation_error_handling() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "message": "Validation failed.",
            "error_code": "validation_error",
            "errors": {
                "from": ["The sender email address is required."],
                "to": ["At least one recipient email address is required."]
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let email = lettr::CreateEmailOptions::new(
        "sender@example.com",
        ["user@example.com"],
        "Test",
    )
    .with_html("<p>Test</p>");

    let err = client.emails.send(email).await.unwrap_err();
    match err {
        lettr::Error::Validation(val_err) => {
            assert_eq!(val_err.message, "Validation failed.");
            assert!(val_err.errors.contains_key("from"));
            assert!(val_err.errors.contains_key("to"));
        }
        other => panic!("Expected Validation error, got: {other:?}"),
    }
}
