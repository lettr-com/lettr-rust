//! Idempotent sends and the two 409s.

use lettr::emails::is_valid_idempotency_key;
use lettr::{CreateEmailOptions, Lettr};

fn email() -> CreateEmailOptions {
    CreateEmailOptions::new("sender@example.com", ["recipient@example.com"], "Hello")
        .with_html("<p>Hi</p>")
}

#[test]
fn accepts_the_documented_key_format() {
    assert!(is_valid_idempotency_key("order-confirmation-12345"));
    assert!(is_valid_idempotency_key("a.b_c-1"));
    assert!(is_valid_idempotency_key("a"));
    assert!(is_valid_idempotency_key(&"a".repeat(255)));
}

#[test]
fn rejects_anything_else() {
    assert!(!is_valid_idempotency_key(""));
    assert!(!is_valid_idempotency_key("order 123"));
    assert!(!is_valid_idempotency_key("order/123"));
    assert!(!is_valid_idempotency_key("order:123"));
    assert!(!is_valid_idempotency_key("order-č"));
    assert!(!is_valid_idempotency_key(&"a".repeat(256)));
}

/// The key travels as a header, so it must never appear in the request body.
#[test]
fn the_key_is_not_serialized_into_the_body() {
    let with_key = serde_json::to_string(&email().with_idempotency_key("order-12345")).unwrap();
    let without_key = serde_json::to_string(&email()).unwrap();

    assert!(!with_key.contains("idempotency"));
    assert!(!with_key.contains("order-12345"));

    // The compatibility guarantee: setting a key changes nothing about the
    // request body a caller was already sending.
    assert_eq!(with_key, without_key);
}

#[test]
fn the_client_exposes_the_folders_service() {
    let client = Lettr::new("test-api-key");

    // Compiles only if the service is wired up.
    let _ = &client.folders;
}
