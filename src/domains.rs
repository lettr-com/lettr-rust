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
    /// Status identifier (e.g. "approved", "pending").
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
    /// Tracking domain, if configured.
    pub tracking_domain: Option<String>,
    /// DNS records for domain verification.
    pub dns: Option<DnsRecords>,
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
}
