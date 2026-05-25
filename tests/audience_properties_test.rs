use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

fn property_json() -> serde_json::Value {
    serde_json::json!({
        "id": "prop-1",
        "name": "first_name",
        "type": "string",
        "fallback_value": "there",
        "created_at": "2024-01-15T10:30:00+00:00"
    })
}

#[tokio::test]
async fn list_audience_properties() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/properties"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "properties": [property_json()],
                "pagination": { "total": 1, "per_page": 20, "current_page": 1, "last_page": 1 }
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .properties
        .list(Default::default())
        .await
        .unwrap();
    assert_eq!(response.properties[0].name, "first_name");
    assert_eq!(
        response.properties[0].property_type,
        lettr::audience::properties::AudiencePropertyType::String
    );
}

#[tokio::test]
async fn create_audience_property() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/properties"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({ "message": "ok", "data": property_json() })),
        )
        .mount(&server)
        .await;

    let prop = client(&server)
        .audience
        .properties
        .create(
            lettr::audience::properties::CreateAudiencePropertyOptions::new(
                "first_name",
                lettr::audience::properties::AudiencePropertyType::String,
            )
            .with_fallback_value("there"),
        )
        .await
        .unwrap();

    assert_eq!(prop.id, "prop-1");
    assert_eq!(prop.fallback_value.as_deref(), Some("there"));
}

#[tokio::test]
async fn get_audience_property() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/properties/prop-1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "message": "ok", "data": property_json() })),
        )
        .mount(&server)
        .await;

    let prop = client(&server)
        .audience
        .properties
        .get("prop-1")
        .await
        .unwrap();
    assert_eq!(prop.name, "first_name");
}

#[tokio::test]
async fn update_audience_property() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/audience/properties/prop-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "id": "prop-1",
                "name": "first_name",
                "type": "string",
                "fallback_value": null,
                "created_at": "2024-01-15T10:30:00+00:00"
            }
        })))
        .mount(&server)
        .await;

    let prop = client(&server)
        .audience
        .properties
        .update(
            "prop-1",
            lettr::audience::properties::UpdateAudiencePropertyOptions::new()
                .clear_fallback_value(),
        )
        .await
        .unwrap();
    assert!(prop.fallback_value.is_none());
}

#[tokio::test]
async fn delete_audience_property() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/audience/properties/prop-1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    client(&server)
        .audience
        .properties
        .delete("prop-1")
        .await
        .unwrap();
}

#[test]
fn create_property_serializes_type() {
    let opts = lettr::audience::properties::CreateAudiencePropertyOptions::new(
        "signup_count",
        lettr::audience::properties::AudiencePropertyType::Number,
    );
    let json = serde_json::to_value(&opts).unwrap();
    assert_eq!(json["name"], "signup_count");
    assert_eq!(json["type"], "number");
    assert!(json.get("fallback_value").is_none());
}

#[test]
fn update_property_clear_serializes_null() {
    let opts = lettr::audience::properties::UpdateAudiencePropertyOptions::new()
        .clear_fallback_value();
    let json = serde_json::to_value(&opts).unwrap();
    assert!(json["fallback_value"].is_null());
}

#[test]
fn update_property_unset_omits_field() {
    let opts = lettr::audience::properties::UpdateAudiencePropertyOptions::new();
    let json = serde_json::to_value(&opts).unwrap();
    assert!(json.get("fallback_value").is_none());
}
