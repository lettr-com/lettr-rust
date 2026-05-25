use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

fn topic_json() -> serde_json::Value {
    serde_json::json!({
        "id": "topic-1",
        "name": "Weekly Digest",
        "description": "Roundup of the week's news",
        "default_subscription": "opt_in",
        "visibility": "public",
        "contacts_count": 42,
        "created_at": "2024-01-15T10:30:00+00:00"
    })
}

#[tokio::test]
async fn list_audience_topics() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/topics"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "topics": [topic_json()],
                "pagination": { "total": 1, "per_page": 20, "current_page": 1, "last_page": 1 }
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .topics
        .list(Default::default())
        .await
        .unwrap();
    assert_eq!(response.topics.len(), 1);
    assert_eq!(response.topics[0].name, "Weekly Digest");
    assert_eq!(
        response.topics[0].default_subscription,
        lettr::audience::topics::AudienceTopicDefaultSubscription::OptIn
    );
    assert_eq!(
        response.topics[0].visibility,
        lettr::audience::topics::AudienceTopicVisibility::Public
    );
}

#[tokio::test]
async fn create_audience_topic() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/topics"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({ "message": "ok", "data": topic_json() })),
        )
        .mount(&server)
        .await;

    let topic = client(&server)
        .audience
        .topics
        .create(
            lettr::audience::topics::CreateAudienceTopicOptions::new("Weekly Digest")
                .with_description("Roundup of the week's news")
                .with_visibility(lettr::audience::topics::AudienceTopicVisibility::Public),
        )
        .await
        .unwrap();

    assert_eq!(topic.id, "topic-1");
}

#[tokio::test]
async fn get_audience_topic() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/topics/topic-1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "message": "ok", "data": topic_json() })),
        )
        .mount(&server)
        .await;

    let topic = client(&server)
        .audience
        .topics
        .get("topic-1")
        .await
        .unwrap();
    assert_eq!(topic.name, "Weekly Digest");
}

#[tokio::test]
async fn update_audience_topic() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/audience/topics/topic-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "id": "topic-1",
                "name": "Updated",
                "description": null,
                "default_subscription": "opt_in",
                "visibility": "private",
                "contacts_count": 42,
                "created_at": "2024-01-15T10:30:00+00:00"
            }
        })))
        .mount(&server)
        .await;

    let topic = client(&server)
        .audience
        .topics
        .update(
            "topic-1",
            lettr::audience::topics::UpdateAudienceTopicOptions::new()
                .with_name("Updated")
                .with_visibility(lettr::audience::topics::AudienceTopicVisibility::Private),
        )
        .await
        .unwrap();
    assert_eq!(topic.name, "Updated");
    assert!(topic.description.is_none());
}

#[tokio::test]
async fn delete_audience_topic() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/audience/topics/topic-1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    client(&server)
        .audience
        .topics
        .delete("topic-1")
        .await
        .unwrap();
}

#[tokio::test]
async fn topic_with_null_created_at_deserializes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/topics/topic-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "id": "topic-1",
                "name": "Legacy",
                "description": null,
                "default_subscription": "opt_in",
                "visibility": "private",
                "contacts_count": 0,
                "created_at": null
            }
        })))
        .mount(&server)
        .await;

    let topic = client(&server)
        .audience
        .topics
        .get("topic-1")
        .await
        .unwrap();
    assert!(topic.created_at.is_none());
    assert!(topic.description.is_none());
}

#[test]
fn update_topic_clear_description_serializes_null() {
    let opts = lettr::audience::topics::UpdateAudienceTopicOptions::new().clear_description();
    let json = serde_json::to_value(&opts).unwrap();
    assert!(json["description"].is_null());
}

#[test]
fn update_topic_set_description_serializes_string() {
    let opts = lettr::audience::topics::UpdateAudienceTopicOptions::new()
        .with_description("New description");
    let json = serde_json::to_value(&opts).unwrap();
    assert_eq!(json["description"], "New description");
}

#[test]
fn update_topic_unset_description_omits_field() {
    let opts = lettr::audience::topics::UpdateAudienceTopicOptions::new().with_name("New name");
    let json = serde_json::to_value(&opts).unwrap();
    assert!(json.get("description").is_none());
    assert_eq!(json["name"], "New name");
}

#[test]
fn create_topic_serialization() {
    let opts = lettr::audience::topics::CreateAudienceTopicOptions::new("Weekly")
        .with_default_subscription(
            lettr::audience::topics::AudienceTopicDefaultSubscription::OptOut,
        )
        .with_visibility(lettr::audience::topics::AudienceTopicVisibility::Public);
    let json = serde_json::to_value(&opts).unwrap();
    assert_eq!(json["name"], "Weekly");
    assert_eq!(json["default_subscription"], "opt_out");
    assert_eq!(json["visibility"], "public");
    assert!(json.get("description").is_none());
}
