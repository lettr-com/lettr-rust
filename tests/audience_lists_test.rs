use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

fn audience_list_json() -> serde_json::Value {
    serde_json::json!({
        "id": "9b9c0e9a-0000-0000-0000-000000000001",
        "name": "Newsletter Subscribers",
        "contacts_count": 1234
    })
}

fn pagination_json() -> serde_json::Value {
    serde_json::json!({
        "total": 1,
        "per_page": 20,
        "current_page": 1,
        "last_page": 1
    })
}

#[tokio::test]
async fn list_audience_lists() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/audience/lists"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Audience lists retrieved successfully.",
            "data": {
                "lists": [audience_list_json()],
                "pagination": pagination_json()
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .lists
        .list(lettr::audience::lists::ListAudienceListsOptions::new().per_page(20))
        .await
        .unwrap();

    assert_eq!(response.lists.len(), 1);
    assert_eq!(response.lists[0].name, "Newsletter Subscribers");
    assert_eq!(response.lists[0].contacts_count, 1234);
    assert_eq!(response.pagination.total, 1);
}

#[tokio::test]
async fn create_audience_list() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/audience/lists"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Audience list created successfully.",
            "data": audience_list_json()
        })))
        .mount(&server)
        .await;

    let list = client(&server)
        .audience
        .lists
        .create(lettr::audience::lists::CreateAudienceListOptions::new(
            "Newsletter Subscribers",
        ))
        .await
        .unwrap();

    assert_eq!(list.id, "9b9c0e9a-0000-0000-0000-000000000001");
}

#[tokio::test]
async fn get_audience_list() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/audience/lists/9b9c0e9a-0000-0000-0000-000000000001",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Audience list retrieved successfully.",
            "data": audience_list_json()
        })))
        .mount(&server)
        .await;

    let list = client(&server)
        .audience
        .lists
        .get("9b9c0e9a-0000-0000-0000-000000000001")
        .await
        .unwrap();

    assert_eq!(list.name, "Newsletter Subscribers");
}

#[tokio::test]
async fn update_audience_list() {
    let server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path(
            "/audience/lists/9b9c0e9a-0000-0000-0000-000000000001",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Audience list updated successfully.",
            "data": {
                "id": "9b9c0e9a-0000-0000-0000-000000000001",
                "name": "Renamed",
                "contacts_count": 1234
            }
        })))
        .mount(&server)
        .await;

    let list = client(&server)
        .audience
        .lists
        .update(
            "9b9c0e9a-0000-0000-0000-000000000001",
            lettr::audience::lists::UpdateAudienceListOptions::new().with_name("Renamed"),
        )
        .await
        .unwrap();

    assert_eq!(list.name, "Renamed");
}

#[tokio::test]
async fn delete_audience_list() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path(
            "/audience/lists/9b9c0e9a-0000-0000-0000-000000000001",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    client(&server)
        .audience
        .lists
        .delete("9b9c0e9a-0000-0000-0000-000000000001")
        .await
        .unwrap();
}

#[tokio::test]
async fn bulk_delete_audience_lists() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/audience/lists/bulk"))
        .and(body_json(serde_json::json!({
            "list_ids": ["a", "b", "c"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Audience lists deleted successfully.",
            "data": { "deleted": 3 }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .lists
        .bulk_delete(lettr::audience::lists::BulkDeleteAudienceListsOptions::new(
            vec!["a".into(), "b".into(), "c".into()],
        ))
        .await
        .unwrap();

    assert_eq!(response.deleted, 3);
}

#[test]
fn bulk_delete_audience_lists_serialization() {
    let opts = lettr::audience::lists::BulkDeleteAudienceListsOptions::new(vec![
        "id-1".into(),
        "id-2".into(),
    ]);
    let json = serde_json::to_value(&opts).unwrap();
    assert_eq!(json["list_ids"][0], "id-1");
    assert_eq!(json["list_ids"][1], "id-2");
}
