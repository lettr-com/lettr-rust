use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> lettr::Lettr {
    lettr::Lettr::with_base_url("test-api-key", &server.uri())
}

#[tokio::test]
async fn list_domains() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/domains"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Domains retrieved successfully.",
            "data": {
                "domains": [
                    {
                        "domain": "example.com",
                        "status": "approved",
                        "status_label": "Approved",
                        "can_send": true,
                        "cname_status": "valid",
                        "dkim_status": "valid",
                        "created_at": "2024-01-15T10:30:00+00:00",
                        "updated_at": "2024-01-16T14:45:00+00:00"
                    }
                ]
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let domains = client.domains.list().await.unwrap();

    assert_eq!(domains.len(), 1);
    assert_eq!(domains[0].domain, "example.com");
    assert_eq!(domains[0].status, lettr::domains::DomainStatus::Approved);
    assert!(domains[0].can_send);
    assert_eq!(
        domains[0].cname_status,
        Some(lettr::domains::DnsVerificationStatus::Valid)
    );
    assert_eq!(
        domains[0].dkim_status,
        Some(lettr::domains::DnsVerificationStatus::Valid)
    );
}

#[tokio::test]
async fn create_domain() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/domains"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "message": "Domain created successfully.",
            "data": {
                "domain": "new.example.com",
                "status": "pending",
                "status_label": "Pending Review",
                "dkim": {
                    "public": "MIGfMA0GCSqGSIb3DQEBA...",
                    "selector": "scph0123",
                    "headers": "from:to:subject:date",
                    "signing_domain": "new.example.com"
                }
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let response = client.domains.create("new.example.com").await.unwrap();

    assert_eq!(response.domain, "new.example.com");
    assert_eq!(response.status, lettr::domains::DomainStatus::Pending);
    let dkim = response.dkim.unwrap();
    assert_eq!(dkim.selector, "scph0123");
    assert_eq!(dkim.signing_domain.as_deref(), Some("new.example.com"));
}

#[tokio::test]
async fn get_domain() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/domains/example.com"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Domain retrieved successfully.",
            "data": {
                "domain": "example.com",
                "status": "approved",
                "status_label": "Approved",
                "can_send": true,
                "cname_status": "valid",
                "dkim_status": "valid",
                "dmarc_status": "valid",
                "spf_status": "valid",
                "is_primary_domain": false,
                "tracking_domain": "tracking.example.com",
                "dns": {
                    "dkim": {
                        "selector": "scph0123",
                        "public": "MIGfMA0GCSqGSIb3DQEBA...",
                        "headers": "from:to:subject:date"
                    }
                },
                "dns_provider": {
                    "provider": "cloudflare",
                    "provider_label": "Cloudflare",
                    "nameservers": ["ns1.cloudflare.com", "ns2.cloudflare.com"],
                    "error": null
                },
                "created_at": "2024-01-15T10:30:00+00:00",
                "updated_at": "2024-01-16T14:45:00+00:00"
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let domain = client.domains.get("example.com").await.unwrap();

    assert_eq!(domain.domain, "example.com");
    assert_eq!(
        domain.dmarc_status,
        Some(lettr::domains::DnsVerificationStatus::Valid)
    );
    assert_eq!(
        domain.spf_status,
        Some(lettr::domains::DnsVerificationStatus::Valid)
    );
    assert!(!domain.is_primary_domain);
    assert_eq!(
        domain.tracking_domain.as_deref(),
        Some("tracking.example.com")
    );

    let dns_provider = domain.dns_provider.unwrap();
    assert_eq!(dns_provider.provider, "cloudflare");
    assert_eq!(dns_provider.nameservers.len(), 2);

    let dns = domain.dns.unwrap();
    let dkim = dns.dkim.unwrap();
    assert_eq!(dkim.selector, "scph0123");
}

#[tokio::test]
async fn delete_domain() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/domains/example.com"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = client(&server);
    client.domains.delete("example.com").await.unwrap();
}

#[tokio::test]
async fn verify_domain() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/domains/example.com/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message": "Domain verification completed.",
            "data": {
                "domain": "example.com",
                "dkim_status": "valid",
                "cname_status": "valid",
                "dmarc_status": "valid",
                "spf_status": "valid",
                "is_primary_domain": false,
                "ownership_verified": "true",
                "dns": {
                    "dkim_record": "v=DKIM1; k=rsa; p=MIGfMA0...",
                    "cname_record": "example.sparkpostmail.com",
                    "dkim_error": null,
                    "cname_error": null,
                    "dmarc_record": "v=DMARC1; p=none",
                    "dmarc_error": null,
                    "spf_record": "v=spf1 include:sparkpostmail.com ~all",
                    "spf_error": null
                },
                "dmarc": {
                    "is_valid": true,
                    "status": "valid",
                    "found_at_domain": "example.com",
                    "record": "v=DMARC1; p=none",
                    "policy": "none",
                    "subdomain_policy": null,
                    "error": null,
                    "covered_by_parent_policy": false
                },
                "spf": {
                    "is_valid": true,
                    "status": "valid",
                    "record": "v=spf1 include:sparkpostmail.com ~all",
                    "error": null,
                    "includes_sparkpost": true
                }
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    let result = client.domains.verify("example.com").await.unwrap();

    assert_eq!(result.domain, "example.com");
    assert_eq!(
        result.dkim_status,
        lettr::domains::DnsVerificationStatus::Valid
    );
    assert_eq!(
        result.cname_status,
        lettr::domains::DnsVerificationStatus::Valid
    );
    assert_eq!(
        result.dmarc_status,
        lettr::domains::DnsVerificationStatus::Valid
    );
    assert_eq!(
        result.spf_status,
        lettr::domains::DnsVerificationStatus::Valid
    );
    assert!(!result.is_primary_domain);

    let dns = result.dns.unwrap();
    assert!(dns.dkim_error.is_none());
    assert!(dns.cname_error.is_none());

    let dmarc = result.dmarc.unwrap();
    assert!(dmarc.is_valid);
    assert_eq!(dmarc.policy, Some(lettr::domains::DmarcPolicy::None));
    assert!(!dmarc.covered_by_parent_policy);

    let spf = result.spf.unwrap();
    assert!(spf.is_valid);
    assert!(spf.includes_sparkpost);
}
