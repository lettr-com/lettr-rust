use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

fn segment_json() -> serde_json::Value {
    serde_json::json!({
        "id": "seg-1",
        "name": "Active subscribers",
        "list_id": "list-1",
        "list_name": "Newsletter",
        "condition_groups": [{
            "conditions": [
                { "field": "status", "operator": "equals", "value": "subscribed" },
                { "field": "verified", "operator": "is_true" }
            ]
        }],
        "cached_contacts_count": 100,
        "created_at": "2024-01-15T10:30:00+00:00"
    })
}

#[tokio::test]
async fn list_audience_segments() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/segments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "segments": [segment_json()],
                "pagination": { "total": 1, "per_page": 20, "current_page": 1, "last_page": 1 }
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .audience
        .segments
        .list(Default::default())
        .await
        .unwrap();
    assert_eq!(response.segments[0].name, "Active subscribers");
    assert_eq!(response.segments[0].condition_groups.len(), 1);
    assert_eq!(
        response.segments[0].condition_groups[0].conditions[0].operator,
        lettr::audience::segments::SegmentOperator::Equals
    );
    assert_eq!(
        response.segments[0].condition_groups[0].conditions[1].operator,
        lettr::audience::segments::SegmentOperator::IsTrue
    );
}

#[tokio::test]
async fn create_audience_segment() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/audience/segments"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({ "message": "ok", "data": segment_json() })),
        )
        .mount(&server)
        .await;

    let conditions = lettr::audience::segments::SegmentConditionsInput::new(vec![
        lettr::audience::segments::SegmentConditionGroup::new(vec![
            lettr::audience::segments::SegmentCondition::new(
                "status",
                lettr::audience::segments::SegmentOperator::Equals,
                "subscribed",
            ),
        ]),
    ]);

    let segment = client(&server)
        .audience
        .segments
        .create(lettr::audience::segments::CreateAudienceSegmentOptions::new(
            "Active subscribers",
            conditions,
        ))
        .await
        .unwrap();

    assert_eq!(segment.id, "seg-1");
}

#[tokio::test]
async fn get_audience_segment() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/audience/segments/seg-1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "message": "ok", "data": segment_json() })),
        )
        .mount(&server)
        .await;

    let segment = client(&server)
        .audience
        .segments
        .get("seg-1")
        .await
        .unwrap();
    assert_eq!(segment.cached_contacts_count, Some(100));
}

#[tokio::test]
async fn update_audience_segment() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/audience/segments/seg-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "ok",
            "data": {
                "id": "seg-1",
                "name": "Renamed",
                "list_id": null,
                "list_name": null,
                "condition_groups": [],
                "cached_contacts_count": null,
                "created_at": "2024-01-15T10:30:00+00:00"
            }
        })))
        .mount(&server)
        .await;

    let segment = client(&server)
        .audience
        .segments
        .update(
            "seg-1",
            lettr::audience::segments::UpdateAudienceSegmentOptions::new()
                .with_name("Renamed")
                .clear_list_id(),
        )
        .await
        .unwrap();
    assert_eq!(segment.name, "Renamed");
    assert!(segment.list_id.is_none());
}

#[tokio::test]
async fn delete_audience_segment() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/audience/segments/seg-1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    client(&server)
        .audience
        .segments
        .delete("seg-1")
        .await
        .unwrap();
}

#[test]
fn create_segment_serialization() {
    let conditions = lettr::audience::segments::SegmentConditionsInput::new(vec![
        lettr::audience::segments::SegmentConditionGroup::new(vec![
            lettr::audience::segments::SegmentCondition::new(
                "email",
                lettr::audience::segments::SegmentOperator::Contains,
                "@example.com",
            ),
            lettr::audience::segments::SegmentCondition::unary(
                "verified",
                lettr::audience::segments::SegmentOperator::IsTrue,
            ),
        ]),
    ]);

    let opts = lettr::audience::segments::CreateAudienceSegmentOptions::new("Example", conditions)
        .with_list_id("list-1");
    let json = serde_json::to_value(&opts).unwrap();

    assert_eq!(json["name"], "Example");
    assert_eq!(json["list_id"], "list-1");
    assert_eq!(json["conditions"]["groups"][0]["conditions"][0]["field"], "email");
    assert_eq!(
        json["conditions"]["groups"][0]["conditions"][0]["operator"],
        "contains"
    );
    assert_eq!(
        json["conditions"]["groups"][0]["conditions"][0]["value"],
        "@example.com"
    );
    assert_eq!(
        json["conditions"]["groups"][0]["conditions"][1]["operator"],
        "is_true"
    );
    // Unary condition: `value` must be absent.
    assert!(json["conditions"]["groups"][0]["conditions"][1]
        .get("value")
        .is_none());
}

#[test]
fn update_segment_clear_list_id_serializes_null() {
    let opts = lettr::audience::segments::UpdateAudienceSegmentOptions::new()
        .with_name("Renamed")
        .clear_list_id();
    let json = serde_json::to_value(&opts).unwrap();
    assert_eq!(json["name"], "Renamed");
    assert!(json["list_id"].is_null());
}
