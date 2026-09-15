//! Smoke tests for the `blocking` client (TPL-2577).
//!
//! Under `blocking`, maybe-async strips `async`/`.await` from the whole client at
//! compile time, so the async suite cannot run against it. These tests call one
//! endpoint per service through the sync client, plus both error shapes, so a
//! broken blocking surface fails CI instead of shipping. Field-by-field coverage
//! stays in the async suite.
//!
//! wiremock is async-only, so each test starts its mock server on a Tokio runtime
//! and then calls the client from the plain test thread: `reqwest::blocking`
//! panics when it is used from inside a runtime.

#![cfg(feature = "blocking")]

use lettr::campaigns::{CampaignStatus, ListCampaignsOptions};
use lettr::folders::ListFoldersOptions;
use lettr::templates::TemplatePurpose;
use tokio::runtime::Runtime;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A mock server plus the runtime it was started on.
///
/// `server` is declared first so it drops before `runtime`.
struct Harness {
    server: MockServer,
    runtime: Runtime,
}

impl Harness {
    fn new() -> Self {
        let runtime = Runtime::new().unwrap();
        let server = runtime.block_on(MockServer::start());
        Self { server, runtime }
    }

    fn mock(&self, http_method: &str, route: &str, status: u16, body: serde_json::Value) {
        self.runtime.block_on(
            Mock::given(method(http_method))
                .and(path(route))
                .respond_with(ResponseTemplate::new(status).set_body_json(body))
                .mount(&self.server),
        );
    }

    fn client(&self) -> lettr::Lettr {
        lettr::Lettr::with_base_url("test-api-key", &self.server.uri())
    }
}

fn pagination_json() -> serde_json::Value {
    serde_json::json!({ "total": 1, "per_page": 20, "current_page": 1, "last_page": 1 })
}

#[test]
fn health() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/health",
        200,
        serde_json::json!({
            "message": "Health check passed.",
            "data": { "status": "ok", "timestamp": "2024-01-15T10:00:00Z" }
        }),
    );

    let response = harness.client().health().unwrap();

    assert_eq!(response.data.status, "ok");
}

#[test]
fn emails_send() {
    let harness = Harness::new();
    harness.mock(
        "POST",
        "/emails",
        200,
        serde_json::json!({
            "message": "Email queued for delivery.",
            "data": { "request_id": "12345678901234567890", "accepted": 1, "rejected": 0 }
        }),
    );

    let email = lettr::CreateEmailOptions::new("sender@example.com", ["user@example.com"], "Hi")
        .with_html("<p>Hi</p>");
    let response = harness.client().emails.send(email).unwrap();

    assert_eq!(response.request_id, "12345678901234567890");
    assert_eq!(response.accepted, 1);
}

#[test]
fn domains_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/domains",
        200,
        serde_json::json!({
            "message": "Domains retrieved successfully.",
            "data": {
                "domains": [{
                    "domain": "example.com",
                    "status": "approved",
                    "status_label": "Approved",
                    "can_send": true,
                    "cname_status": "valid",
                    "dkim_status": "valid",
                    "created_at": "2024-01-15T10:30:00+00:00",
                    "updated_at": "2024-01-16T14:45:00+00:00"
                }]
            }
        }),
    );

    let domains = harness.client().domains.list().unwrap();

    assert_eq!(domains.len(), 1);
    assert_eq!(domains[0].status, lettr::domains::DomainStatus::Approved);
}

#[test]
fn webhooks_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/webhooks",
        200,
        serde_json::json!({
            "message": "Webhooks retrieved successfully.",
            "data": {
                "webhooks": [{
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
                }]
            }
        }),
    );

    let webhooks = harness.client().webhooks.list().unwrap();

    assert_eq!(webhooks.len(), 1);
    assert_eq!(
        webhooks[0].auth_type,
        lettr::webhooks::WebhookAuthType::Basic
    );
}

#[test]
fn templates_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/templates",
        200,
        serde_json::json!({
            "message": "Templates retrieved successfully.",
            "data": {
                "templates": [{
                    "id": 1,
                    "name": "Welcome Email",
                    "slug": "welcome-email",
                    "project_id": 5,
                    "folder_id": 10,
                    "created_at": "2025-01-15T10:00:00+00:00",
                    "updated_at": "2025-01-20T14:30:00+00:00"
                }],
                "pagination": pagination_json()
            }
        }),
    );

    let options = lettr::templates::ListTemplatesOptions::new().per_page(20);
    let response = harness.client().templates.list(options).unwrap();

    assert_eq!(response.templates[0].slug, "welcome-email");
    assert_eq!(response.pagination.total, 1);
}

#[test]
fn projects_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/projects",
        200,
        serde_json::json!({
            "message": "Projects retrieved successfully.",
            "data": {
                "projects": [{
                    "id": 1,
                    "name": "Default Project",
                    "emoji": null,
                    "team_id": 42,
                    "created_at": "2024-01-01T00:00:00+00:00",
                    "updated_at": "2024-01-15T10:00:00+00:00"
                }],
                "pagination": pagination_json()
            }
        }),
    );

    let options = lettr::projects::ListProjectsOptions::new().per_page(20);
    let response = harness.client().projects.list(options).unwrap();

    assert_eq!(response.projects[0].name, "Default Project");
    assert_eq!(response.projects[0].team_id, 42);
}

#[test]
fn folders_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/folders",
        200,
        serde_json::json!({
            "message": "Folders retrieved successfully.",
            "data": {
                "folders": [{
                    "id": 11,
                    "name": "Campaigns",
                    "project_id": 5,
                    "purpose": "campaign",
                    "templates_count": 3,
                    "created_at": "2026-01-15T10:00:00+00:00",
                    "updated_at": "2026-01-20T14:30:00+00:00"
                }],
                "pagination": pagination_json()
            }
        }),
    );

    let response = harness
        .client()
        .folders
        .list(ListFoldersOptions::new())
        .unwrap();

    assert_eq!(response.folders[0].id, 11);
    assert_eq!(response.folders[0].purpose, TemplatePurpose::Campaign);
}

#[test]
fn campaigns_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/campaigns",
        200,
        serde_json::json!({
            "message": "Campaigns retrieved successfully.",
            "data": {
                "campaigns": [{
                    "id": "0193e6a8-1f3a-7c2a-b9e2-1aa1d2e5d3f0",
                    "name": "Spring Sale",
                    "subject": "Big sale!",
                    "from_email": "sender@example.com",
                    "from_name": "Lettr",
                    "reply_to": null,
                    "status": "sent",
                    "scheduled_at": null,
                    "total_recipients": 100,
                    "sent_count": 98,
                    "sent_at": "2026-05-01T09:00:00+00:00",
                    "created_at": "2026-04-30T12:00:00+00:00",
                    "stats": {
                        "injections": 100,
                        "deliveries": 98,
                        "bounces": 2,
                        "spam_complaints": 0,
                        "opens": 50,
                        "unique_opens": 40,
                        "clicks": 20,
                        "unique_clicks": 15,
                        "unsubscribes": 1
                    }
                }],
                "pagination": pagination_json()
            }
        }),
    );

    let response = harness
        .client()
        .campaigns
        .list(ListCampaignsOptions::new())
        .unwrap();

    assert_eq!(response.campaigns[0].status, CampaignStatus::Sent);
    assert_eq!(response.campaigns[0].stats.deliveries, 98);
}

#[test]
fn audience_lists_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/audience/lists",
        200,
        serde_json::json!({
            "message": "Audience lists retrieved successfully.",
            "data": {
                "lists": [{
                    "id": "9b9c0e9a-0000-0000-0000-000000000001",
                    "name": "Newsletter Subscribers",
                    "contacts_count": 1234
                }],
                "pagination": pagination_json()
            }
        }),
    );

    let response = harness
        .client()
        .audience
        .lists
        .list(lettr::audience::lists::ListAudienceListsOptions::new())
        .unwrap();

    assert_eq!(response.lists[0].contacts_count, 1234);
}

#[test]
fn audience_contacts_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/audience/contacts",
        200,
        serde_json::json!({
            "message": "ok",
            "data": {
                "contacts": [{
                    "id": "contact-1",
                    "email": "alice@example.com",
                    "status": "subscribed",
                    "properties": { "first_name": "Alice" },
                    "created_at": "2024-01-15T10:30:00+00:00",
                    "lists": [{ "id": "list-1", "name": "Newsletter" }],
                    "topics": [{ "id": "topic-1", "name": "Weekly" }]
                }],
                "pagination": pagination_json()
            }
        }),
    );

    let response = harness
        .client()
        .audience
        .contacts
        .list(lettr::audience::contacts::ListAudienceContactsOptions::new())
        .unwrap();

    assert_eq!(response.contacts[0].email, "alice@example.com");
    assert_eq!(
        response.contacts[0].status,
        lettr::audience::contacts::AudienceContactStatus::Subscribed
    );
}

#[test]
fn audience_topics_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/audience/topics",
        200,
        serde_json::json!({
            "message": "ok",
            "data": {
                "topics": [{
                    "id": "topic-1",
                    "name": "Weekly Digest",
                    "description": "Roundup of the week's news",
                    "default_subscription": "opt_in",
                    "visibility": "public",
                    "contacts_count": 42,
                    "created_at": "2024-01-15T10:30:00+00:00"
                }],
                "pagination": pagination_json()
            }
        }),
    );

    let response = harness
        .client()
        .audience
        .topics
        .list(Default::default())
        .unwrap();

    assert_eq!(
        response.topics[0].default_subscription,
        lettr::audience::topics::AudienceTopicDefaultSubscription::OptIn
    );
}

#[test]
fn audience_properties_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/audience/properties",
        200,
        serde_json::json!({
            "message": "ok",
            "data": {
                "properties": [{
                    "id": "prop-1",
                    "name": "first_name",
                    "type": "string",
                    "fallback_value": "there",
                    "created_at": "2024-01-15T10:30:00+00:00"
                }],
                "pagination": pagination_json()
            }
        }),
    );

    let response = harness
        .client()
        .audience
        .properties
        .list(Default::default())
        .unwrap();

    assert_eq!(
        response.properties[0].property_type,
        lettr::audience::properties::AudiencePropertyType::String
    );
}

#[test]
fn audience_segments_list() {
    let harness = Harness::new();
    harness.mock(
        "GET",
        "/audience/segments",
        200,
        serde_json::json!({
            "message": "ok",
            "data": {
                "segments": [{
                    "id": "seg-1",
                    "name": "Active subscribers",
                    "list_id": "list-1",
                    "list_name": "Newsletter",
                    "condition_groups": [{
                        "conditions": [
                            { "field": "status", "operator": "equals", "value": "subscribed" }
                        ]
                    }],
                    "cached_contacts_count": 100,
                    "created_at": "2024-01-15T10:30:00+00:00"
                }],
                "pagination": pagination_json()
            }
        }),
    );

    let response = harness
        .client()
        .audience
        .segments
        .list(Default::default())
        .unwrap();

    assert_eq!(
        response.segments[0].condition_groups[0].conditions[0].operator,
        lettr::audience::segments::SegmentOperator::Equals
    );
}

#[test]
fn api_error() {
    let harness = Harness::new();
    harness.mock(
        "POST",
        "/emails",
        400,
        serde_json::json!({
            "message": "The sender domain could not be determined.",
            "error_code": "invalid_domain"
        }),
    );

    let email = lettr::CreateEmailOptions::new("bad@example.com", ["user@example.com"], "Hi")
        .with_html("<p>Hi</p>");
    let err = harness.client().emails.send(email).unwrap_err();

    match err {
        lettr::Error::Api(api_err) => assert_eq!(
            api_err.error_code,
            Some(lettr::error::ErrorCode::InvalidDomain)
        ),
        other => panic!("Expected Api error, got: {other:?}"),
    }
}

#[test]
fn validation_error() {
    let harness = Harness::new();
    harness.mock(
        "POST",
        "/emails",
        422,
        serde_json::json!({
            "message": "Validation failed.",
            "error_code": "validation_error",
            "errors": { "from": ["The sender email address is required."] }
        }),
    );

    let email = lettr::CreateEmailOptions::new("sender@example.com", ["user@example.com"], "Hi")
        .with_html("<p>Hi</p>");
    let err = harness.client().emails.send(email).unwrap_err();

    match err {
        lettr::Error::Validation(val_err) => assert!(val_err.errors.contains_key("from")),
        other => panic!("Expected Validation error, got: {other:?}"),
    }
}
