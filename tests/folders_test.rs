// Async-only: under `blocking` the client methods are synchronous. The blocking
// client is covered by tests/blocking_test.rs.
#![cfg(not(feature = "blocking"))]

use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use lettr::folders::ListFoldersOptions;
use lettr::templates::TemplatePurpose;

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

#[tokio::test]
async fn list_folders() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Folders retrieved successfully.",
            "data": {
                "folders": [
                    {
                        "id": 10,
                        "name": "Emails",
                        "project_id": 5,
                        "purpose": "transactional",
                        "templates_count": 12,
                        "created_at": "2026-01-15T10:00:00+00:00",
                        "updated_at": "2026-01-20T14:30:00+00:00"
                    },
                    {
                        "id": 11,
                        "name": "Campaigns",
                        "project_id": 5,
                        "purpose": "campaign",
                        "templates_count": 3,
                        "created_at": "2026-01-15T10:00:00+00:00",
                        "updated_at": "2026-01-20T14:30:00+00:00"
                    }
                ],
                "pagination": { "total": 2, "per_page": 25, "current_page": 1, "last_page": 1 }
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .folders
        .list(ListFoldersOptions::new())
        .await
        .unwrap();

    assert_eq!(response.folders.len(), 2);

    // The id is the whole point: it is what `with_folder_id` wants, and nothing
    // else in the SDK returns one.
    assert_eq!(response.folders[0].id, 10);
    assert_eq!(response.folders[0].templates_count, 12);
    assert_eq!(response.folders[1].purpose, TemplatePurpose::Campaign);
    assert_eq!(response.pagination.total, 2);
}

#[tokio::test]
async fn list_folders_sends_every_filter() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/folders"))
        .and(query_param("project_id", "5"))
        .and(query_param("purpose", "campaign"))
        .and(query_param("per_page", "50"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Folders retrieved successfully.",
            "data": {
                "folders": [],
                "pagination": { "total": 0, "per_page": 50, "current_page": 2, "last_page": 2 }
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .folders
        .list(
            ListFoldersOptions::new()
                .project_id(5)
                .purpose(TemplatePurpose::Campaign)
                .per_page(50)
                .page(2),
        )
        .await
        .unwrap();

    assert!(response.folders.is_empty());
}

/// An API deployment that predates the fields.
#[tokio::test]
async fn folder_without_purpose_defaults_to_transactional() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/folders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Folders retrieved successfully.",
            "data": {
                "folders": [
                    {
                        "id": 10,
                        "name": "Emails",
                        "project_id": 5,
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
        .folders
        .list(ListFoldersOptions::new())
        .await
        .unwrap();

    assert_eq!(response.folders[0].purpose, TemplatePurpose::Transactional);
    assert_eq!(response.folders[0].templates_count, 0);
}
