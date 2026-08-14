use std::collections::HashMap;

use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

fn contact_json() -> serde_json::Value {
    serde_json::json!({
        "id": "contact-1",
        "email": "alice@example.com",
        "status": "subscribed",
        "properties": { "first_name": "Alice" },
        "created_at": "2024-01-15T10:30:00+00:00",
        "lists": [{ "id": "list-1", "name": "Newsletter" }],
        "topics": [{ "id": "topic-1", "name": "Weekly" }]
    })
}

#[tokio::test]
async fn list_audience_contacts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/contacts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "contacts": [contact_json()],
                "pagination": { "total": 1, "per_page": 20, "current_page": 1, "last_page": 1 }
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .contacts
        .list(
            lettr::audience::contacts::ListAudienceContactsOptions::new()
                .status(lettr::audience::contacts::AudienceContactStatus::Subscribed)
                .per_page(20),
        )
        .await
        .unwrap();

    assert_eq!(response.contacts.len(), 1);
    assert_eq!(response.contacts[0].email, "alice@example.com");
    assert_eq!(
        response.contacts[0].status,
        lettr::audience::contacts::AudienceContactStatus::Subscribed
    );
    assert_eq!(response.contacts[0].lists.len(), 1);
    assert_eq!(response.contacts[0].topics.len(), 1);
}

#[tokio::test]
async fn create_audience_contact() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({ "message": "ok", "data": contact_json() })),
        )
        .mount(&server)
        .await;

    let mut props = HashMap::new();
    props.insert("first_name".into(), "Alice".into());

    let contact = client(&server)
        .audience
        .contacts
        .create(
            lettr::audience::contacts::CreateAudienceContactOptions::new("alice@example.com")
                .with_list_id("list-1")
                .with_properties(props),
        )
        .await
        .unwrap();

    assert_eq!(contact.id, "contact-1");
}

#[tokio::test]
async fn bulk_create_audience_contacts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts/bulk"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "ok",
            "data": { "created": 2, "already_existed": 1 }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .contacts
        .bulk_create(
            lettr::audience::contacts::BulkCreateAudienceContactsOptions::new(vec![
                "a@example.com".into(),
                "b@example.com".into(),
                "c@example.com".into(),
            ])
            .with_list_id("list-1"),
        )
        .await
        .unwrap();

    assert_eq!(response.created, 2);
    assert_eq!(response.already_existed, 1);
}

#[tokio::test]
async fn get_audience_contact() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/contacts/contact-1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "message": "ok", "data": contact_json() })),
        )
        .mount(&server)
        .await;

    let contact = client(&server)
        .audience
        .contacts
        .get("contact-1")
        .await
        .unwrap();
    assert_eq!(contact.email, "alice@example.com");
}

#[tokio::test]
async fn update_audience_contact() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/audience/contacts/contact-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "id": "contact-1",
                "email": "alice@example.com",
                "status": "unsubscribed",
                "properties": {},
                "created_at": "2024-01-15T10:30:00+00:00",
                "lists": [],
                "topics": []
            }
        })))
        .mount(&server)
        .await;

    let mut properties = HashMap::new();
    properties.insert("first_name".into(), None);

    let contact = client(&server)
        .audience
        .contacts
        .update(
            "contact-1",
            lettr::audience::contacts::UpdateAudienceContactOptions::new()
                .with_status(lettr::audience::contacts::UpdateAudienceContactStatus::Unsubscribed)
                .with_properties(properties),
        )
        .await
        .unwrap();

    assert_eq!(
        contact.status,
        lettr::audience::contacts::AudienceContactStatus::Unsubscribed
    );
}

#[tokio::test]
async fn delete_audience_contact() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/audience/contacts/contact-1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    client(&server)
        .audience
        .contacts
        .delete("contact-1")
        .await
        .unwrap();
}

#[tokio::test]
async fn list_contacts_accepts_php_empty_array_for_properties() {
    // The PHP backend serializes an empty associative array as `[]`, not `{}`.
    // The SDK must accept either form for the `properties` field on a contact.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/contacts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "contacts": [{
                    "id": "contact-1",
                    "email": "alice@example.com",
                    "status": "subscribed",
                    "properties": [],
                    "created_at": "2024-01-15T10:30:00+00:00",
                    "lists": [],
                    "topics": []
                }],
                "pagination": { "total": 1, "per_page": 20, "current_page": 1, "last_page": 1 }
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .contacts
        .list(Default::default())
        .await
        .unwrap();

    assert_eq!(response.contacts.len(), 1);
    assert!(response.contacts[0].properties.is_empty());
}

#[tokio::test]
async fn list_contacts_rejects_non_empty_array_for_properties() {
    // A non-empty array is a real shape error and must not be silently swallowed.
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/contacts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "contacts": [{
                    "id": "contact-1",
                    "email": "alice@example.com",
                    "status": "subscribed",
                    "properties": ["unexpected"],
                    "created_at": "2024-01-15T10:30:00+00:00",
                    "lists": [],
                    "topics": []
                }],
                "pagination": { "total": 1, "per_page": 20, "current_page": 1, "last_page": 1 }
            }
        })))
        .mount(&server)
        .await;

    let result = client(&server)
        .audience
        .contacts
        .list(Default::default())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn attach_contact_to_list_encodes_special_characters_in_path() {
    let server = MockServer::start().await;
    // A buggy caller passes an ID that contains '/' and '?'. Without
    // percent-encoding the path would be `/audience/contacts/abc/lists/l1/lists/list-1`
    // — a totally different route. With encoding it becomes
    // `/audience/contacts/abc%2Flists%2Fl1%3Ffoo/lists/list-1`.
    Mock::given(method("POST"))
        .and(path(
            "/audience/contacts/abc%2Flists%2Fl1%3Ffoo/lists/list-1",
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Attached."
        })))
        .mount(&server)
        .await;

    client(&server)
        .audience
        .contacts
        .attach_to_list("abc/lists/l1?foo", "list-1")
        .await
        .unwrap();
}

#[tokio::test]
async fn attach_contact_to_list() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts/contact-1/lists/list-1"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Attached."
        })))
        .mount(&server)
        .await;

    client(&server)
        .audience
        .contacts
        .attach_to_list("contact-1", "list-1")
        .await
        .unwrap();
}

#[tokio::test]
async fn detach_contact_from_list() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/audience/contacts/contact-1/lists/list-1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    client(&server)
        .audience
        .contacts
        .detach_from_list("contact-1", "list-1")
        .await
        .unwrap();
}

#[tokio::test]
async fn bulk_attach_contacts_to_lists() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts/lists/bulk"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": { "attached": 4, "already_attached": 2, "total_pairs": 6 }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .contacts
        .bulk_attach_to_lists(
            lettr::audience::contacts::BulkContactListMembershipOptions::new()
                .with_contact_ids(vec!["c1".into(), "c2".into(), "c3".into()])
                .with_list_ids(vec!["l1".into(), "l2".into()]),
        )
        .await
        .unwrap();

    assert_eq!(response.attached, 4);
    assert_eq!(response.already_attached, 2);
    assert_eq!(response.total_pairs, 6);
}

#[tokio::test]
async fn bulk_detach_contacts_from_lists() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/audience/contacts/lists/bulk"))
        .and(body_json(serde_json::json!({
            "contact_ids": ["c1", "c2", "c3"],
            "list_ids": ["l1", "l2"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": { "detached": 3, "not_present": 3, "total_pairs": 6 }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .contacts
        .bulk_detach_from_lists(
            lettr::audience::contacts::BulkContactListMembershipOptions::new()
                .with_contact_ids(vec!["c1".into(), "c2".into(), "c3".into()])
                .with_list_ids(vec!["l1".into(), "l2".into()]),
        )
        .await
        .unwrap();

    assert_eq!(response.detached, 3);
    assert_eq!(response.not_present, 3);
}

#[tokio::test]
async fn subscribe_contact_to_topic() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts/contact-1/topics/topic-1"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Subscribed."
        })))
        .mount(&server)
        .await;

    client(&server)
        .audience
        .contacts
        .subscribe_to_topic("contact-1", "topic-1")
        .await
        .unwrap();
}

#[tokio::test]
async fn unsubscribe_contact_from_topic() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/audience/contacts/contact-1/topics/topic-1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    client(&server)
        .audience
        .contacts
        .unsubscribe_from_topic("contact-1", "topic-1")
        .await
        .unwrap();
}

#[test]
fn create_contact_with_double_opt_in_serialization() {
    let opts = lettr::audience::contacts::CreateAudienceContactOptions::new("alice@example.com")
        .with_double_opt_in(
            lettr::audience::contacts::DoubleOptInConfig::new(
                "noreply@example.com",
                "Confirm your subscription",
                "confirm-subscription",
                "https://example.com/confirmed",
            )
            .with_from_name("Example Team"),
        );

    let json = serde_json::to_value(&opts).unwrap();
    assert_eq!(json["email"], "alice@example.com");
    assert_eq!(json["double_opt_in"]["from"], "noreply@example.com");
    assert_eq!(json["double_opt_in"]["from_name"], "Example Team");
    assert_eq!(
        json["double_opt_in"]["subject"],
        "Confirm your subscription"
    );
    assert_eq!(
        json["double_opt_in"]["template_slug"],
        "confirm-subscription"
    );
    assert_eq!(
        json["double_opt_in"]["redirect_url"],
        "https://example.com/confirmed"
    );
    assert!(json.get("list_id").is_none());
    assert!(json.get("properties").is_none());
}

#[test]
fn update_contact_clears_property_with_null() {
    let mut properties = HashMap::new();
    properties.insert("first_name".to_string(), None);
    properties.insert("city".to_string(), Some("Prague".to_string()));

    let opts =
        lettr::audience::contacts::UpdateAudienceContactOptions::new().with_properties(properties);
    let json = serde_json::to_value(&opts).unwrap();

    // `None` value must serialize to JSON null (not be skipped) so the server clears the property.
    assert!(json["properties"]["first_name"].is_null());
    assert_eq!(json["properties"]["city"], "Prague");
}

#[test]
fn update_contact_status_serializes_snake_case() {
    let opts = lettr::audience::contacts::UpdateAudienceContactOptions::new()
        .with_status(lettr::audience::contacts::UpdateAudienceContactStatus::Unsubscribed);
    let json = serde_json::to_value(&opts).unwrap();
    assert_eq!(json["status"], "unsubscribed");
}

#[test]
fn bulk_membership_serialization() {
    let opts = lettr::audience::contacts::BulkContactListMembershipOptions::new()
        .with_contact_ids(vec!["c1".into(), "c2".into()])
        .with_list_ids(vec!["l1".into()]);
    let json = serde_json::to_value(&opts).unwrap();
    assert_eq!(json["contact_ids"][0], "c1");
    assert_eq!(json["list_ids"][0], "l1");
    assert_eq!(json["contact_ids"].as_array().unwrap().len(), 2);
}

// ── TPL-2105: bulk contact import ──────────────────────────────────────────

#[test]
fn bulk_create_keeps_the_legacy_payload_byte_identical() {
    // A pre-TPL-2105 call must serialize exactly as it did before: no
    // "contacts" key, and no "update_existing" unless it was asked for.
    let opts = lettr::audience::contacts::BulkCreateAudienceContactsOptions::new(vec![
        "a@example.com".into(),
        "b@example.com".into(),
    ])
    .with_list_id("list-1");

    let json = serde_json::to_value(&opts).unwrap();

    assert_eq!(
        json,
        serde_json::json!({
            "emails": ["a@example.com", "b@example.com"],
            "list_id": "list-1"
        })
    );
}

#[test]
fn bulk_create_serializes_per_contact_rows() {
    let mut row_props = HashMap::new();
    row_props.insert("plan".to_string(), "pro".to_string());

    let opts = lettr::audience::contacts::BulkCreateAudienceContactsOptions::for_contacts(vec![
        lettr::audience::contacts::BulkAudienceContactRow::new("cara@example.com")
            .with_properties(row_props)
            .with_list_ids(vec!["l-vip".into()]),
        // Row-level opt-out must beat the batch-wide opt-in below.
        lettr::audience::contacts::BulkAudienceContactRow::new("dan@example.com").with_topics(
            vec![lettr::audience::contacts::AudienceTopicSubscription::opt_out("t-promos")],
        ),
    ])
    .with_list_ids(vec!["l-everyone".into()])
    .with_topics(vec![
        lettr::audience::contacts::AudienceTopicSubscription::opt_in("t-promos"),
    ])
    .with_update_existing(true);

    let json = serde_json::to_value(&opts).unwrap();

    assert!(
        json.get("emails").is_none(),
        "unexpected emails key: {json}"
    );
    assert_eq!(json["contacts"][0]["email"], "cara@example.com");
    assert_eq!(json["contacts"][0]["properties"]["plan"], "pro");
    assert_eq!(json["contacts"][0]["list_ids"][0], "l-vip");
    // A row with nothing extra carries only its address.
    assert!(json["contacts"][1].get("properties").is_none());
    assert_eq!(json["contacts"][1]["topics"][0]["subscription"], "opt_out");
    assert_eq!(json["list_ids"][0], "l-everyone");
    assert_eq!(json["topics"][0]["subscription"], "opt_in");
    assert_eq!(json["update_existing"], true);
}

#[tokio::test]
async fn bulk_create_with_rows_returns_contact_refs() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts/bulk"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "created": 2,
                "already_existed": 0,
                "updated": 1,
                "error_count": 0,
                "errors": [],
                "contacts": [
                    { "id": "c1", "email": "cara@example.com", "created": true },
                    { "id": "c2", "email": "dan@example.com", "created": false }
                ]
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .contacts
        .bulk_create(
            lettr::audience::contacts::BulkCreateAudienceContactsOptions::for_contacts(vec![
                lettr::audience::contacts::BulkAudienceContactRow::new("cara@example.com"),
                lettr::audience::contacts::BulkAudienceContactRow::new("dan@example.com"),
            ]),
        )
        .await
        .unwrap();

    assert_eq!(response.updated, 1);
    assert!(!response.has_errors());
    // IDs come back in submission order, so no follow-up lookup is needed.
    assert_eq!(response.contact_ids(), vec!["c1", "c2"]);
    assert!(!response.contacts[1].created);
    // id_for is case-insensitive: the API normalizes addresses.
    assert_eq!(response.id_for("  CARA@example.com "), Some("c1"));
    assert_eq!(response.id_for("nobody@example.com"), None);
}

#[tokio::test]
async fn bulk_create_reports_skipped_rows() {
    // Partial success: HTTP 201 with errors populated. The call returns Ok even
    // though one row never landed — that is the trap this pins down.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts/bulk"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Contacts created successfully.",
            "data": {
                "created": 1,
                "already_existed": 0,
                "updated": 0,
                "error_count": 1,
                "errors": [{
                    "index": 1,
                    "email": "not-an-email",
                    "error_code": "invalid_email",
                    "error": "The email address is not valid."
                }],
                "contacts": [{ "id": "c1", "email": "cara@example.com", "created": true }]
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .contacts
        .bulk_create(
            lettr::audience::contacts::BulkCreateAudienceContactsOptions::for_contacts(vec![
                lettr::audience::contacts::BulkAudienceContactRow::new("cara@example.com"),
                lettr::audience::contacts::BulkAudienceContactRow::new("not-an-email"),
            ]),
        )
        .await
        .unwrap();

    assert!(response.has_errors());
    assert_eq!(response.error_count, 1);
    assert_eq!(response.errors[0].index, 1);
    assert_eq!(
        response.errors[0].error_code,
        lettr::audience::contacts::BulkAudienceContactErrorCode::InvalidEmail
    );
    assert_eq!(response.contacts.len(), 1);
}

#[tokio::test]
async fn bulk_create_tolerates_a_pre_tpl_2105_response() {
    // An API deployment older than TPL-2105 answers with just the two counters.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts/bulk"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "ok",
            "data": { "created": 2, "already_existed": 1 }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .contacts
        .bulk_create(
            lettr::audience::contacts::BulkCreateAudienceContactsOptions::new(vec![
                "a@example.com".into(),
            ]),
        )
        .await
        .unwrap();

    assert_eq!(response.updated, 0);
    assert!(!response.has_errors());
    assert!(response.contact_ids().is_empty());
}

#[test]
fn bulk_contact_error_survives_an_unknown_code() {
    // A code added server-side must stay readable rather than failing to parse.
    let response: lettr::audience::contacts::BulkCreateAudienceContactsResponse =
        serde_json::from_value(serde_json::json!({
            "created": 0,
            "already_existed": 0,
            "error_count": 1,
            "errors": [{
                "index": 0,
                "email": null,
                "error_code": "some_future_code",
                "error": "Nope."
            }]
        }))
        .unwrap();

    assert_eq!(
        response.errors[0].error_code,
        lettr::audience::contacts::BulkAudienceContactErrorCode::Unknown("some_future_code".into())
    );
    assert_eq!(
        response.errors[0].error_code.to_string(),
        "some_future_code"
    );
    assert!(response.errors[0].email.is_none());
}

#[tokio::test]
async fn create_duplicate_contact_is_a_conflict() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts"))
        .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
            "message": "A contact with the email jane@example.com already exists.",
            "error_code": "resource_already_exists"
        })))
        .mount(&server)
        .await;

    let error = client(&server)
        .audience
        .contacts
        .create(lettr::audience::contacts::CreateAudienceContactOptions::new("jane@example.com"))
        .await
        .unwrap_err();

    // Previously a 500 send_error, which a retry-on-5xx policy would retry.
    assert!(error.is_contact_already_exists());
    assert_eq!(
        error.error_code(),
        Some(&lettr::error::ErrorCode::ResourceAlreadyExists)
    );
    assert!(matches!(error, lettr::Error::Api(_)));
}

#[tokio::test]
async fn create_other_conflict_is_not_a_duplicate() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts"))
        .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
            "message": "Something else conflicted.",
            "error_code": "some_future_code"
        })))
        .mount(&server)
        .await;

    let error = client(&server)
        .audience
        .contacts
        .create(lettr::audience::contacts::CreateAudienceContactOptions::new("jane@example.com"))
        .await
        .unwrap_err();

    assert!(!error.is_contact_already_exists());
}

#[tokio::test]
async fn bulk_subscribe_contacts_to_topics() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/contacts/topics/bulk"))
        .and(body_json(serde_json::json!({
            "contact_ids": ["c1", "c2"],
            "topic_ids": ["t1", "t2"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": { "subscribed": 3, "already_subscribed": 1, "total_pairs": 4 }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .contacts
        .bulk_subscribe_to_topics(
            lettr::audience::contacts::BulkContactTopicMembershipOptions::new()
                .with_contact_ids(vec!["c1".into(), "c2".into()])
                .with_topic_ids(vec!["t1".into(), "t2".into()]),
        )
        .await
        .unwrap();

    assert_eq!(response.subscribed, 3);
    assert_eq!(response.already_subscribed, 1);
    // 2 contacts × 2 topics — the endpoint works over the Cartesian product.
    assert_eq!(response.total_pairs, 4);
}

#[tokio::test]
async fn bulk_unsubscribe_contacts_from_topics() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/audience/contacts/topics/bulk"))
        // The body is the point: DELETE carries the pairs to remove.
        .and(body_json(serde_json::json!({
            "contact_ids": ["c1", "c2"],
            "topic_ids": ["t1", "t2"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": { "unsubscribed": 2, "total_pairs": 4 }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .contacts
        .bulk_unsubscribe_from_topics(
            lettr::audience::contacts::BulkContactTopicMembershipOptions::new()
                .with_contact_ids(vec!["c1".into(), "c2".into()])
                .with_topic_ids(vec!["t1".into(), "t2".into()]),
        )
        .await
        .unwrap();

    // Pairs that did not exist are ignored, so this is below total_pairs.
    assert_eq!(response.unsubscribed, 2);
    assert_eq!(response.total_pairs, 4);
}
