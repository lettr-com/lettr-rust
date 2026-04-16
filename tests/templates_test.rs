use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

#[tokio::test]
async fn list_templates() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/templates"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Templates retrieved successfully.",
            "data": {
                "templates": [
                    {
                        "id": 1,
                        "name": "Welcome Email",
                        "slug": "welcome-email",
                        "project_id": 5,
                        "folder_id": 10,
                        "created_at": "2025-01-15T10:00:00+00:00",
                        "updated_at": "2025-01-20T14:30:00+00:00"
                    }
                ],
                "pagination": {
                    "total": 1,
                    "per_page": 25,
                    "current_page": 1,
                    "last_page": 1
                }
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let options = lettr::templates::ListTemplatesOptions::new().per_page(25);
    let response = client.templates.list(options).await.unwrap();

    assert_eq!(response.templates.len(), 1);
    assert_eq!(response.templates[0].id, 1);
    assert_eq!(response.templates[0].name, "Welcome Email");
    assert_eq!(response.templates[0].slug, "welcome-email");
    assert_eq!(response.templates[0].project_id, 5);
    assert_eq!(response.pagination.total, 1);
    assert_eq!(response.pagination.per_page, 25);
}

#[tokio::test]
async fn create_template() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/templates"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Template created successfully.",
            "data": {
                "id": 123,
                "name": "Welcome Email",
                "slug": "welcome-email",
                "project_id": 5,
                "folder_id": 10,
                "active_version": 1,
                "merge_tags": [
                    {
                        "key": "first_name",
                        "required": true
                    }
                ],
                "created_at": "2026-01-28T12:00:00+00:00"
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let options = lettr::templates::CreateTemplateOptions::new("Welcome Email")
        .with_html("<h1>Hello {{first_name}}!</h1>")
        .with_project_id(5);

    let result = client.templates.create(options).await.unwrap();
    assert_eq!(result.id, 123);
    assert_eq!(result.slug, "welcome-email");
    assert_eq!(result.active_version, 1);
    assert_eq!(result.merge_tags.len(), 1);
    assert_eq!(result.merge_tags[0].key, "first_name");
    assert!(result.merge_tags[0].required);
}

#[tokio::test]
async fn get_template() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/templates/welcome-email"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Template retrieved successfully.",
            "data": {
                "id": 1,
                "name": "Welcome Email",
                "slug": "welcome-email",
                "project_id": 5,
                "folder_id": 10,
                "active_version": 2,
                "versions_count": 3,
                "html": "<html><body><h1>Welcome!</h1></body></html>",
                "json": null,
                "created_at": "2025-01-15T10:00:00+00:00",
                "updated_at": "2025-01-20T14:30:00+00:00"
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let template = client.templates.get("welcome-email", None).await.unwrap();

    assert_eq!(template.id, 1);
    assert_eq!(template.name, "Welcome Email");
    assert_eq!(template.active_version, Some(2));
    assert_eq!(template.versions_count, 3);
    assert!(template.html.is_some());
    assert!(template.json.is_none());
}

#[tokio::test]
async fn update_template() {
    let server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/templates/welcome-email"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Template updated successfully.",
            "data": {
                "id": 123,
                "name": "Updated Welcome Email",
                "slug": "welcome-email",
                "project_id": 5,
                "folder_id": 10,
                "active_version": 2,
                "merge_tags": [
                    {
                        "key": "name",
                        "required": true,
                        "type": "text"
                    }
                ],
                "created_at": "2026-01-15T10:00:00+00:00",
                "updated_at": "2026-01-28T14:30:00+00:00"
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let options = lettr::templates::UpdateTemplateOptions::new()
        .with_name("Updated Welcome Email")
        .with_html("<h1>Hello {{name}}!</h1>");

    let result = client
        .templates
        .update("welcome-email", options)
        .await
        .unwrap();
    assert_eq!(result.name, "Updated Welcome Email");
    assert_eq!(result.active_version, 2);
    assert_eq!(result.merge_tags[0].key, "name");
    assert_eq!(result.merge_tags[0].merge_tag_type.as_deref(), Some("text"));
}

#[tokio::test]
async fn delete_template() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/templates/welcome-email"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Template deleted successfully."
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    client
        .templates
        .delete("welcome-email", None)
        .await
        .unwrap();
}

#[tokio::test]
async fn get_merge_tags() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/templates/welcome-email/merge-tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Merge tags retrieved successfully.",
            "data": {
                "project_id": 13,
                "template_slug": "welcome-email",
                "version": 2,
                "merge_tags": [
                    {
                        "key": "first_name",
                        "required": true,
                        "type": "text"
                    },
                    {
                        "key": "items",
                        "required": false,
                        "children": [
                            {
                                "key": "item_name",
                                "type": "text"
                            },
                            {
                                "key": "item_price",
                                "type": "number"
                            }
                        ]
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let result = client
        .templates
        .get_merge_tags("welcome-email", None, None)
        .await
        .unwrap();

    assert_eq!(result.project_id, 13);
    assert_eq!(result.template_slug, "welcome-email");
    assert_eq!(result.version, 2);
    assert_eq!(result.merge_tags.len(), 2);

    assert_eq!(result.merge_tags[0].key, "first_name");
    assert!(result.merge_tags[0].required);
    assert_eq!(result.merge_tags[0].merge_tag_type.as_deref(), Some("text"));

    assert_eq!(result.merge_tags[1].key, "items");
    let children = result.merge_tags[1].children.as_ref().unwrap();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].key, "item_name");
    assert_eq!(children[0].merge_tag_type.as_deref(), Some("text"));
    assert_eq!(children[1].key, "item_price");
    assert_eq!(children[1].merge_tag_type.as_deref(), Some("number"));
}

#[tokio::test]
async fn get_template_html() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/templates/html"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "data": {
                "html": "<html><body><h1>Hello {{first_name}}!</h1></body></html>",
                "merge_tags": [
                    {
                        "key": "first_name",
                        "name": "First Name",
                        "required": true
                    }
                ],
                "subject": "Welcome!"
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let result = client.templates.get_html(1, "welcome-email").await.unwrap();

    assert!(result.html.contains("Hello {{first_name}}"));
    assert_eq!(result.merge_tags.len(), 1);
    assert_eq!(result.merge_tags[0].key, "first_name");
    assert_eq!(result.merge_tags[0].name, "First Name");
    assert!(result.merge_tags[0].required);
    assert_eq!(result.subject.as_deref(), Some("Welcome!"));
}

#[test]
fn create_template_serialization() {
    let options = lettr::templates::CreateTemplateOptions::new("Test Template")
        .with_html("<p>Hello</p>")
        .with_project_id(5)
        .with_folder_id(10);

    let json = serde_json::to_value(&options).unwrap();
    assert_eq!(json["name"], "Test Template");
    assert_eq!(json["html"], "<p>Hello</p>");
    assert_eq!(json["project_id"], 5);
    assert_eq!(json["folder_id"], 10);
    assert!(json.get("json").is_none());
}

#[test]
fn update_template_serialization() {
    let options = lettr::templates::UpdateTemplateOptions::new()
        .with_name("New Name")
        .with_html("<p>Updated</p>");

    let json = serde_json::to_value(&options).unwrap();
    assert_eq!(json["name"], "New Name");
    assert_eq!(json["html"], "<p>Updated</p>");
    assert!(json.get("project_id").is_none());
    assert!(json.get("json").is_none());
}
