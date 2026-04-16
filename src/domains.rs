use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Service for the `/domains` endpoints.
#[derive(Clone, Debug)]
pub struct DomainsSvc(pub(crate) Arc<Config>);

impl DomainsSvc {
    /// List all sending domains registered with your account.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let domains = client.domains.list().await?;
    /// for domain in &domains {
    ///     println!("{}: {} (can_send: {})", domain.domain, domain.status, domain.can_send);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list(&self) -> crate::Result<Vec<Domain>> {
        let request = self.0.build(Method::GET, "/domains");
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListDomainsResponseWrapper>().await?;
        Ok(wrapper.data.domains)
    }

    /// Register a new sending domain.
    ///
    /// The domain will be created in a pending state until it is verified and approved.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let result = client.domains.create("example.com").await?;
    /// println!("Domain {} created with status: {}", result.domain, result.status);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn create(&self, domain: &str) -> crate::Result<CreateDomainResponse> {
        let body = CreateDomainRequest {
            domain: domain.to_owned(),
        };
        let request = self.0.build(Method::POST, "/domains").json(&body);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<CreateDomainResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Retrieve details of a single sending domain.
    ///
    /// Returns DNS records, tracking domain configuration, and verification status.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let domain = client.domains.get("example.com").await?;
    /// println!("Status: {}, DKIM: {:?}", domain.status, domain.dkim_status);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn get(&self, domain: &str) -> crate::Result<DomainDetail> {
        let path = format!("/domains/{domain}");
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowDomainResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Delete a sending domain.
    ///
    /// The domain will no longer be available for sending emails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// client.domains.delete("example.com").await?;
    /// println!("Domain deleted.");
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn delete(&self, domain: &str) -> crate::Result<()> {
        let path = format!("/domains/{domain}");
        let request = self.0.build(Method::DELETE, &path);
        self.0.send(request).await?;
        Ok(())
    }

    /// Verify a domain's DNS configuration.
    ///
    /// Triggers a DNS lookup to verify DKIM, CNAME, DMARC, and SPF records.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let result = client.domains.verify("example.com").await?;
    /// println!("DKIM: {}, CNAME: {}", result.dkim_status, result.cname_status);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn verify(&self, domain: &str) -> crate::Result<VerifyDomainResponse> {
        let path = format!("/domains/{domain}/verify");
        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<VerifyDomainResponseWrapper>().await?;
        Ok(wrapper.data)
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct CreateDomainRequest {
    domain: String,
}

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListDomainsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListDomainsData,
}

#[derive(Debug, Deserialize)]
struct ListDomainsData {
    domains: Vec<Domain>,
}

/// A sending domain.
#[derive(Debug, Clone, Deserialize)]
pub struct Domain {
    /// Domain name.
    pub domain: String,
    /// Status identifier (e.g. "approved", "pending", "blocked").
    pub status: String,
    /// Human-readable status label.
    pub status_label: String,
    /// Whether this domain can currently send emails.
    pub can_send: bool,
    /// CNAME record verification status.
    pub cname_status: Option<String>,
    /// DKIM record verification status.
    pub dkim_status: Option<String>,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
struct CreateDomainResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: CreateDomainResponse,
}

/// Response from creating a new domain.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDomainResponse {
    /// Domain name.
    pub domain: String,
    /// Initial status (usually "pending").
    pub status: String,
    /// Human-readable status label.
    pub status_label: String,
    /// DKIM configuration.
    pub dkim: Option<DkimInfo>,
}

/// DKIM signing information for a domain.
#[derive(Debug, Clone, Deserialize)]
pub struct DkimInfo {
    /// DKIM public key.
    pub public: String,
    /// DKIM selector.
    pub selector: String,
    /// DKIM headers configuration.
    pub headers: String,
    /// Domain used for DKIM signing.
    #[serde(default)]
    pub signing_domain: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ShowDomainResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: DomainDetail,
}

/// Detailed domain information including DNS records.
#[derive(Debug, Clone, Deserialize)]
pub struct DomainDetail {
    /// Domain name.
    pub domain: String,
    /// Status identifier.
    pub status: String,
    /// Human-readable status label.
    pub status_label: String,
    /// Whether this domain can currently send emails.
    pub can_send: bool,
    /// CNAME record verification status.
    pub cname_status: Option<String>,
    /// DKIM record verification status.
    pub dkim_status: Option<String>,
    /// DMARC verification status.
    #[serde(default)]
    pub dmarc_status: Option<String>,
    /// SPF verification status.
    #[serde(default)]
    pub spf_status: Option<String>,
    /// Whether this is a primary (apex) sending domain.
    #[serde(default)]
    pub is_primary_domain: Option<bool>,
    /// Tracking domain, if configured.
    pub tracking_domain: Option<String>,
    /// DNS records for domain verification.
    pub dns: Option<DnsRecords>,
    /// Detected DNS provider.
    #[serde(default)]
    pub dns_provider: Option<DnsProvider>,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// DNS records for domain verification.
#[derive(Debug, Clone, Deserialize)]
pub struct DnsRecords {
    /// DKIM DNS record information.
    pub dkim: Option<DkimDnsRecord>,
}

/// DKIM DNS record details.
#[derive(Debug, Clone, Deserialize)]
pub struct DkimDnsRecord {
    /// DKIM selector.
    pub selector: String,
    /// DKIM public key.
    pub public: String,
    /// Headers included in DKIM signature.
    #[serde(default)]
    pub headers: Option<String>,
}

/// Detected DNS provider for a domain.
#[derive(Debug, Clone, Deserialize)]
pub struct DnsProvider {
    /// Machine-readable DNS provider identifier.
    pub provider: String,
    /// Human-readable DNS provider name.
    pub provider_label: String,
    /// Nameservers detected for the domain.
    pub nameservers: Vec<String>,
    /// Error message if DNS provider detection failed.
    pub error: Option<String>,
}

// ── Domain Verification Types ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct VerifyDomainResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: VerifyDomainResponse,
}

/// Response from verifying a domain's DNS configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyDomainResponse {
    /// The domain name.
    pub domain: String,
    /// DKIM verification status.
    pub dkim_status: String,
    /// CNAME verification status.
    pub cname_status: String,
    /// DMARC verification status.
    pub dmarc_status: String,
    /// SPF verification status.
    pub spf_status: String,
    /// Whether this is a primary (apex) sending domain.
    pub is_primary_domain: bool,
    /// Whether domain ownership has been verified.
    #[serde(default)]
    pub ownership_verified: Option<String>,
    /// DNS verification details.
    #[serde(default)]
    pub dns: Option<DomainDnsVerification>,
    /// DMARC validation result.
    #[serde(default)]
    pub dmarc: Option<DmarcValidationResult>,
    /// SPF validation result.
    #[serde(default)]
    pub spf: Option<SpfValidationResult>,
}

/// DNS verification error details.
#[derive(Debug, Clone, Deserialize)]
pub struct DomainDnsVerification {
    /// Found DKIM record value.
    #[serde(default)]
    pub dkim_record: Option<String>,
    /// Found CNAME record value.
    #[serde(default)]
    pub cname_record: Option<String>,
    /// DKIM verification error message.
    #[serde(default)]
    pub dkim_error: Option<String>,
    /// CNAME verification error message.
    #[serde(default)]
    pub cname_error: Option<String>,
    /// Found DMARC record value.
    #[serde(default)]
    pub dmarc_record: Option<String>,
    /// DMARC verification error message.
    #[serde(default)]
    pub dmarc_error: Option<String>,
    /// Found SPF record value.
    #[serde(default)]
    pub spf_record: Option<String>,
    /// SPF verification error message.
    #[serde(default)]
    pub spf_error: Option<String>,
}

/// DMARC validation result.
#[derive(Debug, Clone, Deserialize)]
pub struct DmarcValidationResult {
    /// Whether a valid DMARC record was found.
    pub is_valid: bool,
    /// DMARC validation status.
    pub status: String,
    /// Domain where the DMARC record was located.
    #[serde(default)]
    pub found_at_domain: Option<String>,
    /// Raw DMARC record value.
    #[serde(default)]
    pub record: Option<String>,
    /// DMARC policy tag (`p=`).
    #[serde(default)]
    pub policy: Option<String>,
    /// DMARC subdomain policy tag (`sp=`).
    #[serde(default)]
    pub subdomain_policy: Option<String>,
    /// Validation error message.
    #[serde(default)]
    pub error: Option<String>,
    /// Whether covered by parent domain's subdomain policy.
    pub covered_by_parent_policy: bool,
}

/// SPF validation result.
#[derive(Debug, Clone, Deserialize)]
pub struct SpfValidationResult {
    /// Whether a valid SPF record was found.
    pub is_valid: bool,
    /// SPF validation status.
    pub status: String,
    /// Raw SPF record value.
    #[serde(default)]
    pub record: Option<String>,
    /// Validation error message.
    #[serde(default)]
    pub error: Option<String>,
    /// Whether the SPF record authorises SparkPost.
    pub includes_sparkpost: bool,
}
