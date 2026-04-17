use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

#[tokio::test]
async fn send_email() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Email queued for delivery.",
            "data": {
                "request_id": "12345678901234567890",
                "accepted": 1,
                "rejected": 0
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let email =
        lettr::CreateEmailOptions::new("sender@example.com", ["user@example.com"], "Hello!")
            .with_html("<h1>Welcome!</h1>")
            .with_text("Welcome!")
            .with_from_name("Sender Name")
            .with_cc("cc@example.com")
            .with_bcc("bcc@example.com")
            .with_reply_to("reply@example.com")
            .with_reply_to_name("Reply Name")
            .with_tag("welcome-series")
            .with_header("X-Custom-ID", "abc-123")
            .with_click_tracking(true)
            .with_open_tracking(true)
            .with_transactional(true)
            .with_inline_css(true)
            .with_perform_substitutions(true);

    let response = client.emails.send(email).await.unwrap();
    assert_eq!(response.request_id, "12345678901234567890");
    assert_eq!(response.accepted, 1);
    assert_eq!(response.rejected, 0);
}

#[tokio::test]
async fn send_email_with_template() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Email queued for delivery.",
            "data": {
                "request_id": "99999999999999999999",
                "accepted": 1,
                "rejected": 0
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let email = lettr::CreateEmailOptions::new_with_template(
        "sender@example.com",
        ["user@example.com"],
        "welcome-email",
    )
    .with_substitution("first_name", "John")
    .with_project_id(5);

    let response = client.emails.send(email).await.unwrap();
    assert_eq!(response.request_id, "99999999999999999999");
}

#[tokio::test]
async fn list_emails() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/emails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "message": "Emails retrieved successfully.",
            "data": {
                "events": {
                    "data": [
                        {
                            "event_id": "evt-001",
                            "type": "injection",
                            "timestamp": "2024-01-15T10:30:00Z",
                            "request_id": "req-001",
                            "subject": "Welcome",
                            "friendly_from": "sender@example.com",
                            "rcpt_to": "user@example.com"
                        }
                    ],
                    "total_count": 1,
                    "from": "2024-01-01T00:00:00Z",
                    "to": "2024-01-31T23:59:59Z",
                    "pagination": {
                        "next_cursor": null,
                        "per_page": 25
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let options = lettr::emails::ListEmailsOptions::new()
        .per_page(25)
        .from_date("2024-01-01")
        .to_date("2024-01-31");

    let response = client.emails.list(options).await.unwrap();
    assert_eq!(response.events.data.len(), 1);
    assert_eq!(response.events.total_count, 1);
    assert_eq!(response.events.data[0].event_id, "evt-001");
    assert_eq!(
        response.events.data[0].event_type,
        lettr::emails::EventType::Injection
    );
    assert_eq!(response.events.data[0].subject.as_deref(), Some("Welcome"));
    assert_eq!(response.events.pagination.per_page, 25);
}

#[tokio::test]
async fn get_email_detail() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/emails/12345678901234567890"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Email retrieved successfully.",
            "data": {
                "transmission_id": "12345678901234567890",
                "state": "delivered",
                "from": "sender@example.com",
                "from_name": null,
                "subject": "Welcome to Lettr",
                "recipients": ["recipient@example.com"],
                "num_recipients": 1,
                "scheduled_at": null,
                "events": [
                    {
                        "event_id": "evt-001",
                        "type": "injection",
                        "timestamp": "2024-01-15T10:30:00Z",
                        "request_id": "12345678901234567890",
                        "rcpt_to": "recipient@example.com",
                        "raw_rcpt_to": "recipient@example.com",
                        "recipient_domain": "example.com",
                        "mailbox_provider": "Gmail",
                        "mailbox_provider_region": "Global"
                    },
                    {
                        "event_id": "evt-002",
                        "type": "delivery",
                        "timestamp": "2024-01-15T10:31:00Z",
                        "request_id": "12345678901234567890",
                        "rcpt_to": "recipient@example.com",
                        "raw_rcpt_to": "recipient@example.com",
                        "recipient_domain": "example.com",
                        "mailbox_provider": "Gmail",
                        "mailbox_provider_region": "Global",
                        "queue_time": 1845
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let response = client
        .emails
        .get("12345678901234567890", None, None)
        .await
        .unwrap();

    assert_eq!(response.transmission_id, "12345678901234567890");
    assert_eq!(response.state, lettr::emails::EmailState::Delivered);
    assert_eq!(response.from, "sender@example.com");
    assert_eq!(response.subject, "Welcome to Lettr");
    assert_eq!(response.recipients.len(), 1);
    assert_eq!(response.num_recipients, 1);
    assert!(response.scheduled_at.is_none());
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events[0].event_type,
        lettr::emails::EventType::Injection
    );
    assert_eq!(
        response.events[1].event_type,
        lettr::emails::EventType::Delivery
    );
    assert_eq!(response.events[1].queue_time, Some(1845));
}

#[tokio::test]
async fn list_email_events() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/emails/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Email events retrieved successfully.",
            "data": {
                "events": {
                    "data": [
                        {
                            "event_id": "evt-001",
                            "type": "delivery",
                            "timestamp": "2024-01-15T10:31:00Z",
                            "request_id": "req-001",
                            "rcpt_to": "user@example.com",
                            "raw_rcpt_to": "user@example.com",
                            "recipient_domain": "example.com",
                            "mailbox_provider": "Gmail",
                            "mailbox_provider_region": "Global",
                            "queue_time": 1845,
                            "outbound_tls": "1"
                        },
                        {
                            "event_id": "evt-002",
                            "type": "bounce",
                            "timestamp": "2024-01-15T10:32:00Z",
                            "request_id": "req-002",
                            "rcpt_to": "bad@example.com",
                            "raw_rcpt_to": "bad@example.com",
                            "recipient_domain": "example.com",
                            "mailbox_provider": "Gmail",
                            "mailbox_provider_region": "Global",
                            "bounce_class": 10,
                            "error_code": "550",
                            "reason": "User unknown"
                        }
                    ],
                    "total_count": 2,
                    "from": "2024-01-01T00:00:00Z",
                    "to": "2024-01-31T23:59:59Z",
                    "pagination": {
                        "next_cursor": null,
                        "per_page": 25
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let options = lettr::emails::ListEmailEventsOptions::new()
        .events(vec!["delivery".into(), "bounce".into()])
        .per_page(25);

    let response = client.emails.list_events(options).await.unwrap();
    assert_eq!(response.events.data.len(), 2);
    assert_eq!(response.events.total_count, 2);
    assert_eq!(
        response.events.data[0].event_type,
        lettr::emails::EventType::Delivery
    );
    assert_eq!(response.events.data[0].queue_time, Some(1845));
    assert_eq!(
        response.events.data[1].event_type,
        lettr::emails::EventType::Bounce
    );
    assert_eq!(response.events.data[1].bounce_class, Some(10));
    assert_eq!(response.events.data[1].error_code.as_deref(), Some("550"));
}

#[tokio::test]
async fn schedule_email() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/emails/scheduled"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Email scheduled for delivery.",
            "data": {
                "request_id": "sched-001",
                "accepted": 1,
                "rejected": 0
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let email =
        lettr::CreateEmailOptions::new("sender@example.com", ["user@example.com"], "Scheduled!")
            .with_html("<h1>Scheduled!</h1>");

    let options = lettr::emails::ScheduleEmailOptions::new(email, "2024-01-16T10:00:00Z");
    let response = client.emails.schedule(options).await.unwrap();
    assert_eq!(response.request_id, "sched-001");
    assert_eq!(response.accepted, 1);
}

#[tokio::test]
async fn get_scheduled_email() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/emails/scheduled/12345678901234567890"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Scheduled transmission retrieved successfully.",
            "data": {
                "transmission_id": "12345678901234567890",
                "state": "submitted",
                "scheduled_at": "2024-01-16T10:00:00+00:00",
                "from": "sender@example.com",
                "from_name": "Sender Name",
                "subject": "Scheduled Newsletter",
                "recipients": ["recipient@example.com"],
                "num_recipients": 1,
                "events": []
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let response = client
        .emails
        .get_scheduled("12345678901234567890")
        .await
        .unwrap();

    assert_eq!(response.transmission_id, "12345678901234567890");
    assert_eq!(
        response.state,
        lettr::emails::ScheduledEmailState::Submitted
    );
    assert_eq!(
        response.scheduled_at.as_deref(),
        Some("2024-01-16T10:00:00+00:00")
    );
    assert_eq!(response.from, "sender@example.com");
    assert_eq!(response.from_name.as_deref(), Some("Sender Name"));
    assert_eq!(response.subject, "Scheduled Newsletter");
    assert!(response.events.is_empty());
}

#[tokio::test]
async fn cancel_scheduled_email() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/emails/scheduled/12345678901234567890"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;

    let client = client(&server);
    client
        .emails
        .cancel_scheduled("12345678901234567890")
        .await
        .unwrap();
}

#[tokio::test]
async fn email_event_with_click_data() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/emails/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Email events retrieved successfully.",
            "data": {
                "events": {
                    "data": [
                        {
                            "event_id": "evt-click",
                            "type": "click",
                            "timestamp": "2024-01-15T12:00:00Z",
                            "request_id": "req-001",
                            "rcpt_to": "user@example.com",
                            "raw_rcpt_to": "user@example.com",
                            "recipient_domain": "example.com",
                            "mailbox_provider": "Gmail",
                            "mailbox_provider_region": "Global",
                            "target_link_url": "https://example.com/link",
                            "target_link_name": "Click here",
                            "user_agent": "Mozilla/5.0",
                            "user_agent_parsed": {
                                "agent_family": "Chrome",
                                "os_family": "Windows",
                                "os_version": "10",
                                "is_mobile": false,
                                "is_proxy": false,
                                "is_prefetched": false
                            },
                            "geo_ip": {
                                "country": "CZ",
                                "region": "10",
                                "city": "Prague",
                                "latitude": 50.0883,
                                "longitude": 14.4124,
                                "zip": "110",
                                "postal_code": "110 00"
                            },
                            "ip_address": "104.28.114.11"
                        }
                    ],
                    "total_count": 1,
                    "from": "2024-01-01T00:00:00Z",
                    "to": "2024-01-31T23:59:59Z",
                    "pagination": {
                        "next_cursor": null,
                        "per_page": 25
                    }
                }
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let options = lettr::emails::ListEmailEventsOptions::new().events(vec!["click".into()]);

    let response = client.emails.list_events(options).await.unwrap();
    let event = &response.events.data[0];

    assert_eq!(event.event_type, lettr::emails::EventType::Click);
    assert_eq!(
        event.target_link_url.as_deref(),
        Some("https://example.com/link")
    );
    assert_eq!(event.target_link_name.as_deref(), Some("Click here"));
    assert_eq!(event.ip_address.as_deref(), Some("104.28.114.11"));

    let ua = event.user_agent_parsed.as_ref().unwrap();
    assert_eq!(ua.agent_family.as_deref(), Some("Chrome"));
    assert_eq!(ua.os_family.as_deref(), Some("Windows"));
    assert_eq!(ua.is_mobile, Some(false));

    let geo = event.geo_ip.as_ref().unwrap();
    assert_eq!(geo.country.as_deref(), Some("CZ"));
    assert_eq!(geo.city.as_deref(), Some("Prague"));
    assert_eq!(geo.latitude, Some(50.0883));
}

#[test]
fn create_email_options_serialization() {
    let email =
        lettr::CreateEmailOptions::new("sender@example.com", ["user@example.com"], "Test Subject")
            .with_html("<p>Hello</p>")
            .with_cc("cc@example.com")
            .with_bcc("bcc@example.com")
            .with_reply_to("reply@example.com")
            .with_reply_to_name("Reply Name")
            .with_amp_html("<html amp4email>")
            .with_tag("tag-1")
            .with_header("X-Custom", "value")
            .with_metadata_entry("user_id", "123")
            .with_substitution("name", "John")
            .with_attachment(lettr::Attachment::new(
                "file.pdf",
                "application/pdf",
                "base64==",
            ))
            .with_inline_css(true)
            .with_perform_substitutions(true);

    let json = serde_json::to_value(&email).unwrap();

    assert_eq!(json["from"], "sender@example.com");
    assert_eq!(json["to"][0], "user@example.com");
    assert_eq!(json["subject"], "Test Subject");
    assert_eq!(json["html"], "<p>Hello</p>");
    assert_eq!(json["cc"][0], "cc@example.com");
    assert_eq!(json["bcc"][0], "bcc@example.com");
    assert_eq!(json["reply_to"], "reply@example.com");
    assert_eq!(json["reply_to_name"], "Reply Name");
    assert_eq!(json["amp_html"], "<html amp4email>");
    assert_eq!(json["tag"], "tag-1");
    assert_eq!(json["headers"]["X-Custom"], "value");
    assert_eq!(json["metadata"]["user_id"], "123");
    assert_eq!(json["substitution_data"]["name"], "John");
    assert_eq!(json["attachments"][0]["name"], "file.pdf");
    assert_eq!(json["attachments"][0]["type"], "application/pdf");
    assert_eq!(json["options"]["inline_css"], true);
    assert_eq!(json["options"]["perform_substitutions"], true);
}

#[test]
fn template_email_serialization() {
    let email = lettr::CreateEmailOptions::new_with_template(
        "sender@example.com",
        ["user@example.com"],
        "welcome-email",
    );

    let json = serde_json::to_value(&email).unwrap();

    assert_eq!(json["from"], "sender@example.com");
    assert_eq!(json["template_slug"], "welcome-email");
    assert!(json.get("subject").is_none() || json["subject"].is_null());
}

#[test]
fn schedule_email_serialization() {
    let email =
        lettr::CreateEmailOptions::new("sender@example.com", ["user@example.com"], "Scheduled")
            .with_html("<p>Later</p>");

    let schedule = lettr::emails::ScheduleEmailOptions::new(email, "2024-01-16T10:00:00Z");
    let json = serde_json::to_value(&schedule).unwrap();

    assert_eq!(json["from"], "sender@example.com");
    assert_eq!(json["subject"], "Scheduled");
    assert_eq!(json["scheduled_at"], "2024-01-16T10:00:00Z");
    assert_eq!(json["html"], "<p>Later</p>");
}
