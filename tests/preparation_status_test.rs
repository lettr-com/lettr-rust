use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use lettr::templates::{ListTemplatesOptions, TemplatePreparationStatus, TemplatePurpose};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

fn template(slug: &str, purpose: &str, status: &str) -> serde_json::Value {
    serde_json::json!({
        "id": 1,
        "name": slug,
        "slug": slug,
        "project_id": 5,
        "folder_id": 10,
        "purpose": purpose,
        "preparation_status": status,
        "created_at": "2026-01-15T10:00:00+00:00",
        "updated_at": "2026-01-20T14:30:00+00:00"
    })
}

#[tokio::test]
async fn list_reports_the_preparation_status_of_every_row() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/templates"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Templates retrieved successfully.",
            "data": {
                "templates": [
                    template("ready-one", "transactional", "ready"),
                    template("still-working", "campaign", "pending"),
                    template("gave-up", "transactional", "failed")
                ],
                "pagination": { "total": 3, "per_page": 25, "current_page": 1, "last_page": 1 }
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .templates
        .list(ListTemplatesOptions::new())
        .await
        .unwrap();

    // One list call is the point: a bulk import reconciles everything here
    // instead of a detail call each, all dragging the full HTML payload.
    let statuses: Vec<_> = response
        .templates
        .iter()
        .map(|t| t.preparation_status.clone())
        .collect();

    assert_eq!(
        statuses,
        vec![
            TemplatePreparationStatus::Ready,
            TemplatePreparationStatus::Pending,
            TemplatePreparationStatus::Failed,
        ]
    );

    assert!(response.templates[0].preparation_status.is_settled());
    assert!(!response.templates[1].preparation_status.is_settled());
    assert_eq!(response.templates[1].purpose, TemplatePurpose::Campaign);
}

#[tokio::test]
async fn list_sends_the_folder_and_purpose_filters() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/templates"))
        .and(query_param("folder_id", "10"))
        .and(query_param("purpose", "campaign"))
        .and(query_param("per_page", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Templates retrieved successfully.",
            "data": {
                "templates": [],
                "pagination": { "total": 0, "per_page": 100, "current_page": 1, "last_page": 1 }
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .templates
        .list(
            ListTemplatesOptions::new()
                .folder_id(10)
                .purpose(TemplatePurpose::Campaign)
                .per_page(100),
        )
        .await
        .unwrap();

    assert!(response.templates.is_empty());
}

/// An API deployment that predates the fields had every template with HTML
/// simply usable, so `Ready` is the honest default. `Pending` would look like a
/// stalled queue and hang anything waiting for readiness.
#[tokio::test]
async fn a_template_without_the_fields_reads_as_transactional_and_ready() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/templates"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Templates retrieved successfully.",
            "data": {
                "templates": [
                    {
                        "id": 1,
                        "name": "Legacy",
                        "slug": "legacy",
                        "project_id": 5,
                        "folder_id": 10,
                        "created_at": "2026-01-15T10:00:00+00:00",
                        "updated_at": "2026-01-20T14:30:00+00:00"
                    }
                ],
                "pagination": { "total": 1, "per_page": 25, "current_page": 1, "last_page": 1 }
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .templates
        .list(ListTemplatesOptions::new())
        .await
        .unwrap();

    assert_eq!(
        response.templates[0].purpose,
        TemplatePurpose::Transactional
    );
    assert_eq!(
        response.templates[0].preparation_status,
        TemplatePreparationStatus::Ready
    );
    assert!(response.templates[0].preparation_status.is_settled());
}

/// `is_settled` answers "is what I sent what will go out", not "can I send
/// this" — after an update a pending template is still sendable.
#[test]
fn only_ready_is_settled() {
    assert!(TemplatePreparationStatus::Ready.is_settled());
    assert!(!TemplatePreparationStatus::Pending.is_settled());
    assert!(!TemplatePreparationStatus::Failed.is_settled());
}
