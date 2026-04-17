use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

#[tokio::test]
async fn list_projects() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "message": "Projects retrieved successfully.",
            "data": {
                "projects": [
                    {
                        "id": 1,
                        "name": "Default Project",
                        "emoji": null,
                        "team_id": 42,
                        "created_at": "2024-01-01T00:00:00+00:00",
                        "updated_at": "2024-01-15T10:00:00+00:00"
                    },
                    {
                        "id": 2,
                        "name": "Marketing",
                        "emoji": "📧",
                        "team_id": 42,
                        "created_at": "2024-02-01T00:00:00+00:00",
                        "updated_at": "2024-03-01T00:00:00+00:00"
                    }
                ],
                "pagination": {
                    "total": 2,
                    "per_page": 25,
                    "current_page": 1,
                    "last_page": 1
                }
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let options = lettr::projects::ListProjectsOptions::new().per_page(25);
    let response = client.projects.list(options).await.unwrap();

    assert_eq!(response.projects.len(), 2);
    assert_eq!(response.projects[0].id, 1);
    assert_eq!(response.projects[0].name, "Default Project");
    assert!(response.projects[0].emoji.is_none());
    assert_eq!(response.projects[0].team_id, 42);

    assert_eq!(response.projects[1].id, 2);
    assert_eq!(response.projects[1].name, "Marketing");
    assert_eq!(response.projects[1].emoji.as_deref(), Some("📧"));

    assert_eq!(response.pagination.total, 2);
    assert_eq!(response.pagination.per_page, 25);
    assert_eq!(response.pagination.current_page, 1);
    assert_eq!(response.pagination.last_page, 1);
}

#[tokio::test]
async fn list_projects_with_pagination() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/projects"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "message": "Projects retrieved successfully.",
            "data": {
                "projects": [],
                "pagination": {
                    "total": 0,
                    "per_page": 10,
                    "current_page": 2,
                    "last_page": 1
                }
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let options = lettr::projects::ListProjectsOptions::new()
        .per_page(10)
        .page(2);
    let response = client.projects.list(options).await.unwrap();

    assert!(response.projects.is_empty());
    assert_eq!(response.pagination.per_page, 10);
    assert_eq!(response.pagination.current_page, 2);
}
