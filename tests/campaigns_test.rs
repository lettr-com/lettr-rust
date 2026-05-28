use lettr::campaigns::{
    CampaignEventType, CampaignStatus, ListCampaignEventsOptions, ListCampaignsOptions,
    ScheduleCampaignOptions,
};
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

fn campaign_id() -> &'static str {
    "0193e6a8-1f3a-7c2a-b9e2-1aa1d2e5d3f0"
}

fn campaign_stats_json() -> serde_json::Value {
    serde_json::json!({
        "injections": 100,
        "deliveries": 98,
        "bounces": 2,
        "spam_complaints": 0,
        "opens": 50,
        "unique_opens": 40,
        "clicks": 20,
        "unique_clicks": 15,
        "unsubscribes": 1
    })
}

fn campaign_json() -> serde_json::Value {
    serde_json::json!({
        "id": campaign_id(),
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
        "stats": campaign_stats_json()
    })
}

fn campaign_pagination_json() -> serde_json::Value {
    serde_json::json!({
        "total": 1,
        "per_page": 20,
        "current_page": 1,
        "last_page": 1
    })
}

#[tokio::test]
async fn list_campaigns() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/campaigns"))
        .and(query_param("per_page", "20"))
        .and(query_param("status", "sent"))
        .and(query_param_is_missing("page"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Campaigns retrieved successfully.",
            "data": {
                "campaigns": [campaign_json()],
                "pagination": campaign_pagination_json()
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .campaigns
        .list(
            ListCampaignsOptions::new()
                .per_page(20)
                .status(CampaignStatus::Sent),
        )
        .await
        .unwrap();

    assert_eq!(response.campaigns.len(), 1);
    assert_eq!(response.campaigns[0].name, "Spring Sale");
    assert_eq!(response.campaigns[0].status, CampaignStatus::Sent);
    assert_eq!(response.campaigns[0].sent_count, 98);
    assert_eq!(response.campaigns[0].stats.deliveries, 98);
    assert_eq!(response.campaigns[0].stats.unique_opens, 40);
    assert_eq!(response.pagination.total, 1);
}

#[tokio::test]
async fn get_campaign() {
    let server = MockServer::start().await;

    let mut detail = campaign_json();
    detail["html_content"] = serde_json::Value::String("<h1>Hi</h1>".into());

    Mock::given(method("GET"))
        .and(path(format!("/campaigns/{}", campaign_id())))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Campaign retrieved successfully.",
            "data": detail
        })))
        .mount(&server)
        .await;

    let detail = client(&server).campaigns.get(campaign_id()).await.unwrap();

    assert_eq!(detail.id, campaign_id());
    assert_eq!(detail.html_content.as_deref(), Some("<h1>Hi</h1>"));
    assert_eq!(detail.stats.injections, 100);
}

#[tokio::test]
async fn list_campaign_events() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/campaigns/{}/events", campaign_id())))
        .and(query_param("event_type", "open"))
        .and(query_param("cursor", "prev-cursor"))
        .and(query_param("limit", "50"))
        .and(query_param_is_missing("email"))
        .and(query_param_is_missing("start_date"))
        .and(query_param_is_missing("end_date"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Campaign events retrieved successfully.",
            "data": {
                "events": [{
                    "event_id": "92356829",
                    "event_type": "open",
                    "email": "jane@example.com",
                    "timestamp": "2026-05-01T12:30:00+00:00",
                    "user_agent": "Mozilla/5.0"
                }],
                "next_cursor": "cursor-abc"
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .campaigns
        .list_events(
            campaign_id(),
            ListCampaignEventsOptions::new()
                .event_type(CampaignEventType::Open)
                .limit(50)
                .cursor("prev-cursor"),
        )
        .await
        .unwrap();

    assert_eq!(response.events.len(), 1);
    assert_eq!(response.events[0].event_type, CampaignEventType::Open);
    assert_eq!(response.events[0].email, "jane@example.com");
    assert_eq!(
        response.events[0].user_agent.as_deref(),
        Some("Mozilla/5.0")
    );
    assert_eq!(response.next_cursor.as_deref(), Some("cursor-abc"));
}

#[tokio::test]
async fn list_campaign_events_with_null_cursor() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(format!("/campaigns/{}/events", campaign_id())))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Campaign events retrieved successfully.",
            "data": {
                "events": [],
                "next_cursor": null
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .campaigns
        .list_events(campaign_id(), ListCampaignEventsOptions::new())
        .await
        .unwrap();

    assert!(response.events.is_empty());
    assert!(response.next_cursor.is_none());
}

#[tokio::test]
async fn send_campaign() {
    let server = MockServer::start().await;

    let mut sending = campaign_json();
    sending["status"] = serde_json::Value::String("preparing".into());

    Mock::given(method("POST"))
        .and(path(format!("/campaigns/{}/send", campaign_id())))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "message": "Campaign sending started.",
            "data": sending
        })))
        .mount(&server)
        .await;

    let result = client(&server).campaigns.send(campaign_id()).await.unwrap();

    let campaign = result.expect("data should be present");
    assert_eq!(campaign.status, CampaignStatus::Preparing);
}

#[tokio::test]
async fn send_campaign_without_data() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path(format!("/campaigns/{}/send", campaign_id())))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "message": "Campaign sending started."
        })))
        .mount(&server)
        .await;

    let result = client(&server).campaigns.send(campaign_id()).await.unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn schedule_campaign() {
    let server = MockServer::start().await;

    let mut scheduled = campaign_json();
    scheduled["status"] = serde_json::Value::String("scheduled".into());
    scheduled["scheduled_at"] = serde_json::Value::String("2026-06-01T09:00:00+00:00".into());

    Mock::given(method("POST"))
        .and(path(format!("/campaigns/{}/schedule", campaign_id())))
        .and(body_json(serde_json::json!({
            "scheduled_at": "2026-06-01T09:00:00+00:00"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Campaign scheduled for delivery.",
            "data": scheduled
        })))
        .mount(&server)
        .await;

    let result = client(&server)
        .campaigns
        .schedule(
            campaign_id(),
            ScheduleCampaignOptions::new("2026-06-01T09:00:00+00:00"),
        )
        .await
        .unwrap();

    let campaign = result.expect("data should be present");
    assert_eq!(campaign.status, CampaignStatus::Scheduled);
    assert_eq!(
        campaign.scheduled_at.as_deref(),
        Some("2026-06-01T09:00:00+00:00")
    );
}

#[tokio::test]
async fn unschedule_campaign() {
    let server = MockServer::start().await;

    let mut drafted = campaign_json();
    drafted["status"] = serde_json::Value::String("draft".into());
    drafted["scheduled_at"] = serde_json::Value::Null;

    Mock::given(method("POST"))
        .and(path(format!("/campaigns/{}/unschedule", campaign_id())))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Campaign unscheduled.",
            "data": drafted
        })))
        .mount(&server)
        .await;

    let result = client(&server)
        .campaigns
        .unschedule(campaign_id())
        .await
        .unwrap();

    let campaign = result.expect("data should be present");
    assert_eq!(campaign.status, CampaignStatus::Draft);
    assert!(campaign.scheduled_at.is_none());
}

#[test]
fn schedule_campaign_serialization() {
    let opts = ScheduleCampaignOptions::new("2026-06-01T09:00:00+00:00");
    let json = serde_json::to_value(&opts).unwrap();
    assert_eq!(json["scheduled_at"], "2026-06-01T09:00:00+00:00");
    assert_eq!(json.as_object().unwrap().len(), 1);
}
