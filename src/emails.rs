use std::collections::HashMap;
use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Service for the `/emails` endpoints.
#[derive(Clone, Debug)]
pub struct EmailsSvc(pub(crate) Arc<Config>);

impl EmailsSvc {
    /// Send a transactional email.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::{Lettr, CreateEmailOptions};
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let email = CreateEmailOptions::new("sender@example.com", ["user@example.com"], "Hello!")
    ///     .with_html("<h1>Welcome!</h1>")
    ///     .with_text("Welcome!");
    ///
    /// let response = client.emails.send(email).await?;
    /// println!("Request ID: {}", response.request_id);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn send(&self, email: CreateEmailOptions) -> crate::Result<SendEmailResponse> {
        let request = self.0.build(Method::POST, "/emails").json(&email);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<SendEmailResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Retrieve a list of sent emails with optional filtering and pagination.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::emails::ListEmailsOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let options = ListEmailsOptions::new().per_page(10);
    /// let response = client.emails.list(options).await?;
    ///
    /// for email in &response.results {
    ///     println!("{}: {}", email.rcpt_to, email.subject);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list(&self, options: ListEmailsOptions) -> crate::Result<ListEmailsResponse> {
        let mut request = self.0.build(Method::GET, "/emails");

        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }
        if let Some(ref cursor) = options.cursor {
            request = request.query(&[("cursor", cursor.as_str())]);
        }
        if let Some(ref recipients) = options.recipients {
            request = request.query(&[("recipients", recipients.as_str())]);
        }
        if let Some(ref from) = options.from {
            request = request.query(&[("from", from.as_str())]);
        }
        if let Some(ref to) = options.to {
            request = request.query(&[("to", to.as_str())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListEmailsResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Retrieve all events for a specific email by its request ID.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let details = client.emails.get("request-id-here").await?;
    /// for event in &details.results {
    ///     println!("{}: {}", event.event_type, event.timestamp);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn get(&self, request_id: &str) -> crate::Result<GetEmailResponse> {
        let path = format!("/emails/{request_id}");
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<GetEmailResponseWrapper>().await?;
        Ok(wrapper.data)
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for sending an email via the Lettr API.
///
/// Use the builder methods to construct the email step by step.
///
/// At minimum, `from`, `to`, `subject`, and either `html` or `text` must be provided.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct CreateEmailOptions {
    /// Sender email address.
    from: String,

    /// Sender display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    from_name: Option<String>,

    /// Recipient email addresses.
    to: Vec<String>,

    /// Email subject.
    subject: String,

    /// HTML body.
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<String>,

    /// Plain text body.
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,

    /// Reply-to email addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<Vec<String>>,

    /// Template slug for sending with a pre-defined template.
    #[serde(skip_serializing_if = "Option::is_none")]
    template_slug: Option<String>,

    /// Template version number.
    #[serde(skip_serializing_if = "Option::is_none")]
    template_version: Option<u32>,

    /// Project ID for template lookup.
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<u64>,

    /// Substitution data for template personalization.
    #[serde(skip_serializing_if = "Option::is_none")]
    substitution_data: Option<HashMap<String, serde_json::Value>>,

    /// Custom metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, serde_json::Value>>,

    /// File attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<Attachment>>,

    /// Tracking and delivery options.
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<EmailOptions>,
}

impl CreateEmailOptions {
    /// Creates a new [`CreateEmailOptions`].
    ///
    /// - `from`: Sender email address.
    /// - `to`: Recipient email addresses.
    /// - `subject`: Email subject line.
    ///
    /// # Example
    ///
    /// ```
    /// use lettr::CreateEmailOptions;
    ///
    /// let email = CreateEmailOptions::new(
    ///     "sender@example.com",
    ///     ["recipient@example.com"],
    ///     "Hello World",
    /// )
    /// .with_html("<h1>Hello!</h1>")
    /// .with_text("Hello!");
    /// ```
    pub fn new<T, A>(from: impl Into<String>, to: T, subject: impl Into<String>) -> Self
    where
        T: IntoIterator<Item = A>,
        A: Into<String>,
    {
        Self {
            from: from.into(),
            from_name: None,
            to: to.into_iter().map(Into::into).collect(),
            subject: subject.into(),
            html: None,
            text: None,
            reply_to: None,
            template_slug: None,
            template_version: None,
            project_id: None,
            substitution_data: None,
            metadata: None,
            attachments: None,
            options: None,
        }
    }

    /// Sets the sender display name.
    #[inline]
    pub fn with_from_name(mut self, name: impl Into<String>) -> Self {
        self.from_name = Some(name.into());
        self
    }

    /// Sets the HTML body of the email.
    #[inline]
    pub fn with_html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    /// Sets the plain text body of the email.
    #[inline]
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Adds a reply-to email address.
    #[inline]
    pub fn with_reply_to(mut self, address: impl Into<String>) -> Self {
        self.reply_to
            .get_or_insert_with(Vec::new)
            .push(address.into());
        self
    }

    /// Sets the template slug for sending with a pre-defined template.
    #[inline]
    pub fn with_template(mut self, slug: impl Into<String>) -> Self {
        self.template_slug = Some(slug.into());
        self
    }

    /// Sets the template version.
    #[inline]
    pub fn with_template_version(mut self, version: u32) -> Self {
        self.template_version = Some(version);
        self
    }

    /// Sets the project ID for template lookup.
    #[inline]
    pub fn with_project_id(mut self, project_id: u64) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Adds a substitution data key-value pair for template personalization.
    #[inline]
    pub fn with_substitution(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.substitution_data
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Sets all substitution data at once.
    #[inline]
    pub fn with_substitution_data(mut self, data: HashMap<String, serde_json::Value>) -> Self {
        self.substitution_data = Some(data);
        self
    }

    /// Adds a metadata key-value pair.
    #[inline]
    pub fn with_metadata_entry(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Sets all metadata at once.
    #[inline]
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Adds a file attachment.
    #[inline]
    pub fn with_attachment(mut self, attachment: Attachment) -> Self {
        self.attachments
            .get_or_insert_with(Vec::new)
            .push(attachment);
        self
    }

    /// Enables or disables click tracking.
    #[inline]
    pub fn with_click_tracking(mut self, enabled: bool) -> Self {
        self.options
            .get_or_insert_with(EmailOptions::default)
            .click_tracking = Some(enabled);
        self
    }

    /// Enables or disables open tracking.
    #[inline]
    pub fn with_open_tracking(mut self, enabled: bool) -> Self {
        self.options
            .get_or_insert_with(EmailOptions::default)
            .open_tracking = Some(enabled);
        self
    }

    /// Sets whether the email is transactional.
    #[inline]
    pub fn with_transactional(mut self, transactional: bool) -> Self {
        self.options
            .get_or_insert_with(EmailOptions::default)
            .transactional = Some(transactional);
        self
    }
}

/// Tracking and delivery options for an email.
#[must_use]
#[derive(Debug, Default, Clone, Serialize)]
pub struct EmailOptions {
    /// Enable click tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_tracking: Option<bool>,

    /// Enable open tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_tracking: Option<bool>,

    /// Mark as transactional email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactional: Option<bool>,
}

/// A file attachment for an email.
///
/// Attachments must be base64-encoded.
///
/// # Example
///
/// ```
/// use lettr::Attachment;
///
/// let attachment = Attachment::new("invoice.pdf", "application/pdf", "base64data...");
/// ```
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct Attachment {
    /// Filename of the attachment.
    pub name: String,
    /// MIME type (e.g. `"application/pdf"`).
    #[serde(rename = "type")]
    pub content_type: String,
    /// Base64-encoded file content.
    pub data: String,
}

impl Attachment {
    /// Creates a new [`Attachment`].
    pub fn new(
        name: impl Into<String>,
        content_type: impl Into<String>,
        data: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            content_type: content_type.into(),
            data: data.into(),
        }
    }
}

/// Options for listing sent emails.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListEmailsOptions {
    per_page: Option<u32>,
    cursor: Option<String>,
    recipients: Option<String>,
    from: Option<String>,
    to: Option<String>,
}

impl ListEmailsOptions {
    /// Creates new [`ListEmailsOptions`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of results per page (1-100).
    #[inline]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Sets the pagination cursor from a previous response.
    #[inline]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Filters by recipient email address.
    #[inline]
    pub fn recipients(mut self, recipients: impl Into<String>) -> Self {
        self.recipients = Some(recipients.into());
        self
    }

    /// Filters emails sent on or after this date (ISO 8601 format).
    #[inline]
    pub fn from_date(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Filters emails sent on or before this date (ISO 8601 format).
    #[inline]
    pub fn to_date(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }
}

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SendEmailResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: SendEmailResponse,
}

/// Successful response from sending an email.
#[derive(Debug, Clone, Deserialize)]
pub struct SendEmailResponse {
    /// Unique request ID for the transmission.
    pub request_id: String,
    /// Number of accepted recipients.
    pub accepted: u32,
    /// Number of rejected recipients.
    pub rejected: u32,
}

#[derive(Debug, Deserialize)]
struct ListEmailsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListEmailsResponse,
}

/// Response from listing sent emails.
#[derive(Debug, Clone, Deserialize)]
pub struct ListEmailsResponse {
    /// List of email events.
    pub results: Vec<EmailEvent>,
    /// Total number of matching emails.
    pub total_count: u64,
    /// Pagination information.
    pub pagination: Pagination,
}

/// Pagination metadata for list responses.
#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    /// Cursor for fetching the next page, if available.
    pub next_cursor: Option<String>,
    /// Number of results per page.
    pub per_page: u32,
}

#[derive(Debug, Deserialize)]
struct GetEmailResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: GetEmailResponse,
}

/// Response from getting email details.
#[derive(Debug, Clone, Deserialize)]
pub struct GetEmailResponse {
    /// List of events for this email.
    pub results: Vec<EmailEventDetail>,
    /// Total number of events.
    pub total_count: u64,
}

/// A sent email event (returned from list endpoint).
#[derive(Debug, Clone, Deserialize)]
pub struct EmailEvent {
    /// Unique event ID.
    pub event_id: String,
    /// Timestamp of the event.
    pub timestamp: String,
    /// Transmission request ID.
    pub request_id: String,
    /// Message ID.
    pub message_id: String,
    /// Email subject.
    pub subject: String,
    /// Sender email address.
    pub friendly_from: String,
    /// Sending domain.
    pub sending_domain: String,
    /// Recipient email address.
    pub rcpt_to: String,
    /// Raw recipient email address.
    pub raw_rcpt_to: String,
    /// Recipient domain.
    pub recipient_domain: String,
    /// Mailbox provider (e.g. "gmail").
    #[serde(default)]
    pub mailbox_provider: Option<String>,
    /// Mailbox provider region.
    #[serde(default)]
    pub mailbox_provider_region: Option<String>,
    /// Sending IP address.
    #[serde(default)]
    pub sending_ip: Option<String>,
    /// Whether click tracking is enabled.
    #[serde(default)]
    pub click_tracking: bool,
    /// Whether open tracking is enabled.
    #[serde(default)]
    pub open_tracking: bool,
    /// Whether this is a transactional email.
    #[serde(default)]
    pub transactional: bool,
    /// Message size in bytes.
    #[serde(default)]
    pub msg_size: Option<u64>,
    /// Injection time.
    #[serde(default)]
    pub injection_time: Option<String>,
    /// Recipient metadata.
    #[serde(default)]
    pub rcpt_meta: Option<serde_json::Value>,
}

/// Detailed email event (returned from get endpoint).
#[derive(Debug, Clone, Deserialize)]
pub struct EmailEventDetail {
    /// Unique event ID.
    pub event_id: String,
    /// Event type (e.g. "injection", "delivery", "bounce").
    #[serde(rename = "type")]
    pub event_type: String,
    /// Timestamp of the event.
    pub timestamp: String,
    /// Transmission request ID.
    pub request_id: String,
    /// Message ID.
    pub message_id: String,
    /// Email subject.
    pub subject: String,
    /// Sender email address.
    pub friendly_from: String,
    /// Sending domain.
    pub sending_domain: String,
    /// Recipient email address.
    pub rcpt_to: String,
    /// Raw recipient email address.
    pub raw_rcpt_to: String,
    /// Recipient domain.
    pub recipient_domain: String,
    /// Mailbox provider.
    #[serde(default)]
    pub mailbox_provider: Option<String>,
    /// Mailbox provider region.
    #[serde(default)]
    pub mailbox_provider_region: Option<String>,
    /// Sending IP address.
    #[serde(default)]
    pub sending_ip: Option<String>,
    /// Whether click tracking is enabled.
    #[serde(default)]
    pub click_tracking: bool,
    /// Whether open tracking is enabled.
    #[serde(default)]
    pub open_tracking: bool,
    /// Whether this is a transactional email.
    #[serde(default)]
    pub transactional: bool,
    /// Message size in bytes.
    #[serde(default)]
    pub msg_size: Option<u64>,
    /// Injection time.
    #[serde(default)]
    pub injection_time: Option<String>,
    /// Bounce or failure reason.
    #[serde(default)]
    pub reason: Option<String>,
    /// Raw reason string.
    #[serde(default)]
    pub raw_reason: Option<String>,
    /// Error code for bounce/failure.
    #[serde(default)]
    pub error_code: Option<String>,
    /// Recipient metadata.
    #[serde(default)]
    pub rcpt_meta: Option<serde_json::Value>,
}
