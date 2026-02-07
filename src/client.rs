use std::sync::Arc;

use crate::config::Config;
use crate::domains::DomainsSvc;
use crate::emails::EmailsSvc;
use crate::templates::TemplatesSvc;
use crate::webhooks::WebhooksSvc;

/// The Lettr API client.
///
/// Create a client using [`Lettr::new`] with your API key, then access
/// the various API services through the public fields.
///
/// # Example
///
/// ```rust,no_run
/// use lettr::{Lettr, CreateEmailOptions};
///
/// # async fn run() -> lettr::Result<()> {
/// let client = Lettr::new("your-api-key");
///
/// let email = CreateEmailOptions::new("sender@example.com", ["user@example.com"], "Hello!")
///     .with_html("<h1>Hello World!</h1>");
///
/// let response = client.emails.send(email).await?;
/// println!("Request ID: {}", response.request_id);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Lettr {
    /// Email sending, listing, and retrieval.
    pub emails: EmailsSvc,
    /// Domain management.
    pub domains: DomainsSvc,
    /// Webhook listing and retrieval.
    pub webhooks: WebhooksSvc,
    /// Template listing and creation.
    pub templates: TemplatesSvc,

    config: Arc<Config>,
}

impl Lettr {
    /// Creates a new [`Lettr`] client with the given API key.
    ///
    /// # Panics
    ///
    /// Panics if the API key contains non-ASCII characters.
    #[must_use]
    pub fn new(api_key: &str) -> Self {
        let config = Arc::new(Config::new(api_key));

        Self {
            emails: EmailsSvc(Arc::clone(&config)),
            domains: DomainsSvc(Arc::clone(&config)),
            webhooks: WebhooksSvc(Arc::clone(&config)),
            templates: TemplatesSvc(Arc::clone(&config)),
            config,
        }
    }

    /// Creates a new [`Lettr`] client from the `LETTR_API_KEY` environment variable.
    ///
    /// # Panics
    ///
    /// Panics if the environment variable is not set.
    #[must_use]
    pub fn from_env() -> Self {
        let api_key =
            std::env::var("LETTR_API_KEY").expect("LETTR_API_KEY environment variable not set");
        Self::new(&api_key)
    }

    /// Check the health of the Lettr API.
    ///
    /// This endpoint does not require authentication.
    #[maybe_async::maybe_async]
    pub async fn health(&self) -> crate::Result<HealthResponse> {
        let request = self.config.build(reqwest::Method::GET, "/health");
        let response = self.config.send(request).await?;
        let body = response.json::<HealthResponse>().await?;
        Ok(body)
    }

    /// Validate the API key and return associated team information.
    #[maybe_async::maybe_async]
    pub async fn auth_check(&self) -> crate::Result<AuthCheckResponse> {
        let request = self.config.build(reqwest::Method::GET, "/auth/check");
        let response = self.config.send(request).await?;
        let body = response.json::<AuthCheckResponse>().await?;
        Ok(body)
    }
}

/// Response from the health check endpoint.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HealthResponse {
    /// Status message.
    pub message: String,
    /// Health check data.
    pub data: HealthData,
}

/// Health check data.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct HealthData {
    /// Health status (e.g., "ok").
    pub status: String,
    /// Timestamp of the health check.
    pub timestamp: String,
}

/// Response from the auth check endpoint.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthCheckResponse {
    /// Status message.
    pub message: String,
    /// Auth check data.
    pub data: AuthCheckData,
}

/// Auth check data.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthCheckData {
    /// The team ID associated with the API key.
    pub team_id: i64,
    /// Timestamp of the auth check.
    pub timestamp: String,
}
