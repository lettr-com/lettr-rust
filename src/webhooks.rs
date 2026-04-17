use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::config::Config;

// ── Enum Types ────────────────────────────────────────────────────────────

/// Authentication type for a webhook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WebhookAuthType {
    None,
    Basic,
    Oauth2,
    /// An unknown auth type not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for WebhookAuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Basic => write!(f, "basic"),
            Self::Oauth2 => write!(f, "oauth2"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Last delivery status of a webhook.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WebhookStatus {
    Success,
    Failure,
    /// An unknown status not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for WebhookStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Mode for selecting which events trigger a webhook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WebhookEventsMode {
    All,
    Selected,
}

impl std::fmt::Display for WebhookEventsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Selected => write!(f, "selected"),
        }
    }
}

/// Webhook event type constants.
///
/// Use these when creating or updating webhooks with `WebhookEventsMode::Selected`.
///
/// Note: The engagement event names use `engagament` (missing an 'e') to match the API spec.
pub mod event_types {
    // Message events
    pub const INJECTION: &str = "message.injection";
    pub const DELIVERY: &str = "message.delivery";
    pub const BOUNCE: &str = "message.bounce";
    pub const DELAY: &str = "message.delay";
    pub const OUT_OF_BAND: &str = "message.out_of_band";
    pub const SPAM_COMPLAINT: &str = "message.spam_complaint";
    pub const POLICY_REJECTION: &str = "message.policy_rejection";

    // Engagement events (note: "engagament" spelling matches the API)
    pub const CLICK: &str = "engagament.click";
    pub const OPEN: &str = "engagament.open";
    pub const INITIAL_OPEN: &str = "engagament.initial_open";
    pub const AMP_CLICK: &str = "engagament.amp_click";
    pub const AMP_OPEN: &str = "engagament.amp_open";
    pub const AMP_INITIAL_OPEN: &str = "engagament.amp_initial_open";

    // Generation events
    pub const GENERATION_FAILURE: &str = "generation.generation_failure";
    pub const GENERATION_REJECTION: &str = "generation.generation_rejection";

    // Unsubscribe events
    pub const LIST_UNSUBSCRIBE: &str = "unsubscribe.list_unsubscribe";
    pub const LINK_UNSUBSCRIBE: &str = "unsubscribe.link_unsubscribe";

    // Relay events
    pub const RELAY_INJECTION: &str = "relay.relay_injection";
    pub const RELAY_REJECTION: &str = "relay.relay_rejection";
    pub const RELAY_DELIVERY: &str = "relay.relay_delivery";
    pub const RELAY_TEMPFAIL: &str = "relay.relay_tempfail";
    pub const RELAY_PERMFAIL: &str = "relay.relay_permfail";
}

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

    /// Create a new webhook.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::webhooks::{CreateWebhookOptions, WebhookAuthType, WebhookEventsMode};
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let options = CreateWebhookOptions::new(
    ///     "Order Notifications",
    ///     "https://example.com/webhook",
    ///     WebhookAuthType::None,
    ///     WebhookEventsMode::All,
    /// );
    /// let webhook = client.webhooks.create(options).await?;
    /// println!("Webhook created: {}", webhook.id);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn create(&self, options: CreateWebhookOptions) -> crate::Result<Webhook> {
        let request = self.0.build(Method::POST, "/webhooks").json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowWebhookResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Update an existing webhook.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::webhooks::UpdateWebhookOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let options = UpdateWebhookOptions::new()
    ///     .with_name("Updated Webhook")
    ///     .with_active(false);
    /// let webhook = client.webhooks.update("webhook-abc123", options).await?;
    /// println!("Updated: {}", webhook.name);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        webhook_id: &str,
        options: UpdateWebhookOptions,
    ) -> crate::Result<Webhook> {
        let path = format!("/webhooks/{webhook_id}");
        let request = self.0.build(Method::PUT, &path).json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowWebhookResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Delete a webhook.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// client.webhooks.delete("webhook-abc123").await?;
    /// println!("Webhook deleted.");
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn delete(&self, webhook_id: &str) -> crate::Result<()> {
        let path = format!("/webhooks/{webhook_id}");
        let request = self.0.build(Method::DELETE, &path);
        self.0.send(request).await?;
        Ok(())
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for creating a webhook.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhookOptions {
    /// Name of the webhook.
    name: String,
    /// URL where webhook events will be sent.
    url: String,
    /// Authentication type.
    auth_type: WebhookAuthType,
    /// Whether to receive all events or only selected ones.
    events_mode: WebhookEventsMode,
    /// List of event types to receive (required when events_mode is "selected").
    #[serde(skip_serializing_if = "Option::is_none")]
    events: Option<Vec<String>>,
    /// Username for basic authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_username: Option<String>,
    /// Password for basic authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_password: Option<String>,
    /// OAuth2 client ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_client_id: Option<String>,
    /// OAuth2 client secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_client_secret: Option<String>,
    /// OAuth2 token URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_token_url: Option<String>,
}

impl CreateWebhookOptions {
    /// Creates a new [`CreateWebhookOptions`].
    pub fn new(
        name: impl Into<String>,
        url: impl Into<String>,
        auth_type: WebhookAuthType,
        events_mode: WebhookEventsMode,
    ) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            auth_type,
            events_mode,
            events: None,
            auth_username: None,
            auth_password: None,
            oauth_client_id: None,
            oauth_client_secret: None,
            oauth_token_url: None,
        }
    }

    /// Sets the event types to receive.
    #[inline]
    pub fn with_events(mut self, events: Vec<String>) -> Self {
        self.events = Some(events);
        self
    }

    /// Sets basic auth credentials.
    #[inline]
    pub fn with_basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.auth_username = Some(username.into());
        self.auth_password = Some(password.into());
        self
    }

    /// Sets OAuth2 credentials.
    #[inline]
    pub fn with_oauth2(
        mut self,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        token_url: impl Into<String>,
    ) -> Self {
        self.oauth_client_id = Some(client_id.into());
        self.oauth_client_secret = Some(client_secret.into());
        self.oauth_token_url = Some(token_url.into());
        self
    }
}

/// Options for updating a webhook. Only provided fields will be updated.
#[must_use]
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateWebhookOptions {
    /// Name of the webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// URL where webhook events will be sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    /// Authentication type.
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_type: Option<WebhookAuthType>,
    /// Username for basic authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_username: Option<String>,
    /// Password for basic authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_password: Option<String>,
    /// OAuth2 token URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_token_url: Option<String>,
    /// OAuth2 client ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_client_id: Option<String>,
    /// OAuth2 client secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_client_secret: Option<String>,
    /// Event types that trigger the webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    events: Option<Vec<String>>,
    /// Whether the webhook is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<bool>,
}

impl UpdateWebhookOptions {
    /// Creates new [`UpdateWebhookOptions`] with no fields set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the webhook name.
    #[inline]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the webhook target URL.
    #[inline]
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Sets the authentication type.
    #[inline]
    pub fn with_auth_type(mut self, auth_type: WebhookAuthType) -> Self {
        self.auth_type = Some(auth_type);
        self
    }

    /// Sets basic auth credentials.
    #[inline]
    pub fn with_basic_auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.auth_username = Some(username.into());
        self.auth_password = Some(password.into());
        self
    }

    /// Sets OAuth2 credentials.
    #[inline]
    pub fn with_oauth2(
        mut self,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        token_url: impl Into<String>,
    ) -> Self {
        self.oauth_client_id = Some(client_id.into());
        self.oauth_client_secret = Some(client_secret.into());
        self.oauth_token_url = Some(token_url.into());
        self
    }

    /// Sets the event types.
    #[inline]
    pub fn with_events(mut self, events: Vec<String>) -> Self {
        self.events = Some(events);
        self
    }

    /// Sets whether the webhook is active.
    #[inline]
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
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
    /// Authentication type.
    pub auth_type: WebhookAuthType,
    /// Whether authentication credentials are configured.
    pub has_auth_credentials: bool,
    /// Timestamp of the last successful delivery.
    pub last_successful_at: Option<String>,
    /// Timestamp of the last failed delivery.
    pub last_failure_at: Option<String>,
    /// Last delivery status.
    pub last_status: Option<WebhookStatus>,
}
