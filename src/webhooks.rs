use std::sync::Arc;

use reqwest::Method;
use serde::Deserialize;

use crate::config::Config;

/// Service for the `/webhooks` endpoints.
#[derive(Clone, Debug)]
pub struct WebhooksSvc(pub(crate) Arc<Config>);

impl WebhooksSvc {
    /// List all webhooks configured for your account.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let webhooks = client.webhooks.list().await?;
    /// for webhook in &webhooks {
    ///     println!("{}: {} (enabled: {})", webhook.id, webhook.name, webhook.enabled);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list(&self) -> crate::Result<Vec<Webhook>> {
        let request = self.0.build(Method::GET, "/webhooks");
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListWebhooksResponseWrapper>().await?;
        Ok(wrapper.data.webhooks)
    }

    /// Retrieve details of a single webhook.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let webhook = client.webhooks.get("webhook-abc123").await?;
    /// println!("URL: {}, Status: {:?}", webhook.url, webhook.last_status);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn get(&self, webhook_id: &str) -> crate::Result<Webhook> {
        let path = format!("/webhooks/{webhook_id}");
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowWebhookResponseWrapper>().await?;
        Ok(wrapper.data)
    }
}

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListWebhooksResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListWebhooksData,
}

#[derive(Debug, Deserialize)]
struct ListWebhooksData {
    webhooks: Vec<Webhook>,
}

#[derive(Debug, Deserialize)]
struct ShowWebhookResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: Webhook,
}

/// A configured webhook.
#[derive(Debug, Clone, Deserialize)]
pub struct Webhook {
    /// Unique webhook ID.
    pub id: String,
    /// Webhook name.
    pub name: String,
    /// Destination URL.
    pub url: String,
    /// Whether the webhook is enabled.
    pub enabled: bool,
    /// Event types this webhook subscribes to.
    pub event_types: Option<Vec<String>>,
    /// Authentication type (e.g. "basic", "none").
    pub auth_type: String,
    /// Whether authentication credentials are configured.
    pub has_auth_credentials: bool,
    /// Timestamp of the last successful delivery.
    pub last_successful_at: Option<String>,
    /// Timestamp of the last failed delivery.
    pub last_failure_at: Option<String>,
    /// Last delivery status (e.g. "success", "failure").
    pub last_status: Option<String>,
}
