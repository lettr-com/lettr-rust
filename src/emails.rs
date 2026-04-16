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
    /// for email in &response.events.data {
    ///     println!("{}: {:?}", email.event_id, email.subject);
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

    /// Retrieve detailed events for a specific email by its request ID.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let detail = client.emails.get("request-id-here", None, None).await?;
    /// println!("State: {}, Recipients: {}", detail.state, detail.num_recipients);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn get(
        &self,
        request_id: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> crate::Result<GetEmailResponse> {
        let path = format!("/emails/{request_id}");
        let mut request = self.0.build(Method::GET, &path);

        if let Some(from) = from {
            request = request.query(&[("from", from)]);
        }
        if let Some(to) = to {
            request = request.query(&[("to", to)]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<GetEmailResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// List email events with optional filtering and pagination.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::emails::ListEmailEventsOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let options = ListEmailEventsOptions::new()
    ///     .events(vec!["delivery".into(), "bounce".into()])
    ///     .per_page(10);
    /// let response = client.emails.list_events(options).await?;
    ///
    /// for event in &response.events.data {
    ///     println!("{}: {}", event.event_type, event.timestamp);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list_events(
        &self,
        options: ListEmailEventsOptions,
    ) -> crate::Result<ListEmailEventsResponse> {
        let mut request = self.0.build(Method::GET, "/emails/events");

        if let Some(ref events) = options.events {
            for event in events {
                request = request.query(&[("events[]", event.as_str())]);
            }
        }
        if let Some(ref recipients) = options.recipients {
            for recipient in recipients {
                request = request.query(&[("recipients[]", recipient.as_str())]);
            }
        }
        if let Some(ref from) = options.from {
            request = request.query(&[("from", from.as_str())]);
        }
        if let Some(ref to) = options.to {
            request = request.query(&[("to", to.as_str())]);
        }
        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }
        if let Some(ref cursor) = options.cursor {
            request = request.query(&[("cursor", cursor.as_str())]);
        }
        if let Some(ref transmissions) = options.transmissions {
            request = request.query(&[("transmissions", transmissions.as_str())]);
        }
        if let Some(ref bounce_classes) = options.bounce_classes {
            request = request.query(&[("bounce_classes", bounce_classes.as_str())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListEmailEventsResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Schedule an email for future delivery.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::{Lettr, CreateEmailOptions};
    /// # use lettr::emails::ScheduleEmailOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let email = CreateEmailOptions::new("sender@example.com", ["user@example.com"], "Hello!")
    ///     .with_html("<h1>Scheduled!</h1>");
    ///
    /// let options = ScheduleEmailOptions::new(email, "2024-01-16T10:00:00Z");
    /// let response = client.emails.schedule(options).await?;
    /// println!("Request ID: {}", response.request_id);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn schedule(
        &self,
        options: ScheduleEmailOptions,
    ) -> crate::Result<SendEmailResponse> {
        let request = self.0.build(Method::POST, "/emails/scheduled").json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<SendEmailResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Retrieve details of a scheduled email.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let scheduled = client.emails.get_scheduled("12345678901234567890").await?;
    /// println!("State: {}, Scheduled at: {:?}", scheduled.state, scheduled.scheduled_at);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn get_scheduled(
        &self,
        transmission_id: &str,
    ) -> crate::Result<ScheduledTransmission> {
        let path = format!("/emails/scheduled/{transmission_id}");
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ScheduledTransmissionResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Cancel a scheduled email.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// client.emails.cancel_scheduled("12345678901234567890").await?;
    /// println!("Scheduled email cancelled.");
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn cancel_scheduled(&self, transmission_id: &str) -> crate::Result<()> {
        let path = format!("/emails/scheduled/{transmission_id}");
        let request = self.0.build(Method::DELETE, &path);
        self.0.send(request).await?;
        Ok(())
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for sending an email via the Lettr API.
///
/// Use the builder methods to construct the email step by step.
///
/// At minimum, `from`, `to`, and either `html`, `text`, or `template_slug` must be provided.
/// The `subject` is required unless `template_slug` is provided.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct CreateEmailOptions {
    /// Sender email address.
    from: String,

    /// Sender display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    from_name: Option<String>,

    /// Email subject. Optional when using a template.
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,

    /// Recipient email addresses.
    to: Vec<String>,

    /// CC recipient email addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    cc: Option<Vec<String>>,

    /// BCC recipient email addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    bcc: Option<Vec<String>>,

    /// Reply-to email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to: Option<String>,

    /// Reply-to display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    reply_to_name: Option<String>,

    /// HTML body.
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<String>,

    /// Plain text body.
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,

    /// AMP HTML content for supported email clients.
    #[serde(skip_serializing_if = "Option::is_none")]
    amp_html: Option<String>,

    /// Project ID for template lookup.
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<u64>,

    /// Template slug for sending with a pre-defined template.
    #[serde(skip_serializing_if = "Option::is_none")]
    template_slug: Option<String>,

    /// Template version number.
    #[serde(skip_serializing_if = "Option::is_none")]
    template_version: Option<u32>,

    /// Tag for tracking and analytics.
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,

    /// Custom metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,

    /// Custom email headers.
    #[serde(skip_serializing_if = "Option::is_none")]
    headers: Option<HashMap<String, String>>,

    /// Substitution data for template personalization.
    #[serde(skip_serializing_if = "Option::is_none")]
    substitution_data: Option<HashMap<String, serde_json::Value>>,

    /// Tracking and delivery options.
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<EmailOptions>,

    /// File attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    attachments: Option<Vec<Attachment>>,
}

impl CreateEmailOptions {
    /// Creates a new [`CreateEmailOptions`] with a subject.
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
            subject: Some(subject.into()),
            to: to.into_iter().map(Into::into).collect(),
            cc: None,
            bcc: None,
            reply_to: None,
            reply_to_name: None,
            html: None,
            text: None,
            amp_html: None,
            project_id: None,
            template_slug: None,
            template_version: None,
            tag: None,
            metadata: None,
            headers: None,
            substitution_data: None,
            options: None,
            attachments: None,
        }
    }

    /// Creates a new [`CreateEmailOptions`] for sending with a template (no subject required).
    ///
    /// # Example
    ///
    /// ```
    /// use lettr::CreateEmailOptions;
    ///
    /// let email = CreateEmailOptions::new_with_template(
    ///     "sender@example.com",
    ///     ["recipient@example.com"],
    ///     "welcome-email",
    /// )
    /// .with_substitution("first_name", "John");
    /// ```
    pub fn new_with_template<T, A>(
        from: impl Into<String>,
        to: T,
        template_slug: impl Into<String>,
    ) -> Self
    where
        T: IntoIterator<Item = A>,
        A: Into<String>,
    {
        Self {
            from: from.into(),
            from_name: None,
            subject: None,
            to: to.into_iter().map(Into::into).collect(),
            cc: None,
            bcc: None,
            reply_to: None,
            reply_to_name: None,
            html: None,
            text: None,
            amp_html: None,
            project_id: None,
            template_slug: Some(template_slug.into()),
            template_version: None,
            tag: None,
            metadata: None,
            headers: None,
            substitution_data: None,
            options: None,
            attachments: None,
        }
    }

    /// Sets the sender display name.
    #[inline]
    pub fn with_from_name(mut self, name: impl Into<String>) -> Self {
        self.from_name = Some(name.into());
        self
    }

    /// Sets the email subject.
    #[inline]
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Adds a CC recipient email address.
    #[inline]
    pub fn with_cc(mut self, address: impl Into<String>) -> Self {
        self.cc.get_or_insert_with(Vec::new).push(address.into());
        self
    }

    /// Adds a BCC recipient email address.
    #[inline]
    pub fn with_bcc(mut self, address: impl Into<String>) -> Self {
        self.bcc.get_or_insert_with(Vec::new).push(address.into());
        self
    }

    /// Sets the reply-to email address.
    #[inline]
    pub fn with_reply_to(mut self, address: impl Into<String>) -> Self {
        self.reply_to = Some(address.into());
        self
    }

    /// Sets the reply-to display name.
    #[inline]
    pub fn with_reply_to_name(mut self, name: impl Into<String>) -> Self {
        self.reply_to_name = Some(name.into());
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

    /// Sets the AMP HTML content for supported email clients.
    #[inline]
    pub fn with_amp_html(mut self, amp_html: impl Into<String>) -> Self {
        self.amp_html = Some(amp_html.into());
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

    /// Sets the tag for tracking and analytics.
    #[inline]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Adds a substitution data key-value pair for template personalization.
    #[inline]
    pub fn with_substitution(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
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
    pub fn with_metadata_entry(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.metadata
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Sets all metadata at once.
    #[inline]
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Adds a custom email header.
    #[inline]
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Sets all custom email headers at once.
    #[inline]
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
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

    /// Enables or disables CSS inlining.
    #[inline]
    pub fn with_inline_css(mut self, enabled: bool) -> Self {
        self.options
            .get_or_insert_with(EmailOptions::default)
            .inline_css = Some(enabled);
        self
    }

    /// Enables or disables variable substitutions in content.
    #[inline]
    pub fn with_perform_substitutions(mut self, enabled: bool) -> Self {
        self.options
            .get_or_insert_with(EmailOptions::default)
            .perform_substitutions = Some(enabled);
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

    /// Inline CSS styles in HTML content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_css: Option<bool>,

    /// Perform variable substitutions in content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perform_substitutions: Option<bool>,
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

    /// Filters emails sent on or after this date (YYYY-MM-DD format).
    #[inline]
    pub fn from_date(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Filters emails sent on or before this date (YYYY-MM-DD format).
    #[inline]
    pub fn to_date(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }
}

/// Options for listing email events.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListEmailEventsOptions {
    events: Option<Vec<String>>,
    recipients: Option<Vec<String>>,
    from: Option<String>,
    to: Option<String>,
    per_page: Option<u32>,
    cursor: Option<String>,
    transmissions: Option<String>,
    bounce_classes: Option<String>,
}

impl ListEmailEventsOptions {
    /// Creates new [`ListEmailEventsOptions`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by event types (e.g. `["delivery", "bounce"]`).
    #[inline]
    pub fn events(mut self, events: Vec<String>) -> Self {
        self.events = Some(events);
        self
    }

    /// Filters by recipient email addresses.
    #[inline]
    pub fn recipients(mut self, recipients: Vec<String>) -> Self {
        self.recipients = Some(recipients);
        self
    }

    /// Filters events from this date (YYYY-MM-DD format).
    #[inline]
    pub fn from_date(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Filters events until this date (YYYY-MM-DD format).
    #[inline]
    pub fn to_date(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
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

    /// Filters by transmission ID.
    #[inline]
    pub fn transmissions(mut self, transmissions: impl Into<String>) -> Self {
        self.transmissions = Some(transmissions.into());
        self
    }

    /// Filters by bounce classes (comma-separated, e.g. `"10,30"`).
    #[inline]
    pub fn bounce_classes(mut self, bounce_classes: impl Into<String>) -> Self {
        self.bounce_classes = Some(bounce_classes.into());
        self
    }
}

/// Options for scheduling an email for future delivery.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct ScheduleEmailOptions {
    /// The email to schedule.
    #[serde(flatten)]
    pub email: CreateEmailOptions,

    /// The UTC date/time when the email should be sent (ISO 8601).
    /// Must be at least 5 minutes in the future and within 3 days.
    pub scheduled_at: String,
}

impl ScheduleEmailOptions {
    /// Creates a new [`ScheduleEmailOptions`].
    pub fn new(email: CreateEmailOptions, scheduled_at: impl Into<String>) -> Self {
        Self {
            email,
            scheduled_at: scheduled_at.into(),
        }
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
    /// Email events data with pagination.
    pub events: SentEmailEventsData,
}

/// Sent email events data container.
#[derive(Debug, Clone, Deserialize)]
pub struct SentEmailEventsData {
    /// List of sent email events.
    pub data: Vec<SentEmailListItem>,
    /// Total number of matching emails.
    pub total_count: u64,
    /// Start of the query date range.
    pub from: Option<String>,
    /// End of the query date range.
    pub to: Option<String>,
    /// Pagination information.
    pub pagination: Pagination,
}

/// A sent email list item (returned from list endpoint).
#[derive(Debug, Clone, Deserialize)]
pub struct SentEmailListItem {
    /// Unique event ID.
    pub event_id: String,
    /// Event type (e.g. "injection").
    #[serde(rename = "type")]
    pub event_type: String,
    /// Timestamp of the event.
    pub timestamp: String,
    /// Transmission request ID.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Message ID.
    #[serde(default)]
    pub message_id: Option<String>,
    /// Email subject.
    #[serde(default)]
    pub subject: Option<String>,
    /// Sender email address.
    #[serde(default)]
    pub friendly_from: Option<String>,
    /// Sending domain.
    #[serde(default)]
    pub sending_domain: Option<String>,
    /// Recipient email address.
    #[serde(default)]
    pub rcpt_to: Option<String>,
    /// Raw recipient email address.
    #[serde(default)]
    pub raw_rcpt_to: Option<String>,
    /// Recipient domain.
    #[serde(default)]
    pub recipient_domain: Option<String>,
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
    pub click_tracking: Option<bool>,
    /// Whether open tracking is enabled.
    #[serde(default)]
    pub open_tracking: Option<bool>,
    /// Whether this is a transactional email.
    #[serde(default)]
    pub transactional: Option<bool>,
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
    /// Unique transmission ID.
    pub transmission_id: String,
    /// Derived delivery state (e.g. "scheduled", "delivered", "bounced", "failed").
    pub state: String,
    /// Sender email address.
    pub from: String,
    /// Sender display name.
    #[serde(default)]
    pub from_name: Option<String>,
    /// Email subject line.
    pub subject: String,
    /// List of recipient email addresses.
    pub recipients: Vec<String>,
    /// Total number of recipients.
    pub num_recipients: u32,
    /// Scheduled delivery time, if applicable.
    #[serde(default)]
    pub scheduled_at: Option<String>,
    /// Delivery events for this email.
    pub events: Vec<EmailEvent>,
}

#[derive(Debug, Deserialize)]
struct ListEmailEventsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListEmailEventsResponse,
}

/// Response from listing email events.
#[derive(Debug, Clone, Deserialize)]
pub struct ListEmailEventsResponse {
    /// Email events data with pagination.
    pub events: EmailEventsData,
}

/// Email events data container.
#[derive(Debug, Clone, Deserialize)]
pub struct EmailEventsData {
    /// List of email events.
    pub data: Vec<EmailEvent>,
    /// Total number of events matching the query.
    pub total_count: u64,
    /// Start of the date range.
    pub from: Option<String>,
    /// End of the date range.
    pub to: Option<String>,
    /// Pagination information.
    pub pagination: Pagination,
}

/// An email event with type-specific fields.
///
/// The `event_type` field determines which additional fields are present.
/// Fields specific to certain event types will be `None` for other types.
#[derive(Debug, Clone, Deserialize)]
pub struct EmailEvent {
    // ── Common required fields ──
    /// Unique event ID.
    pub event_id: String,
    /// Event type (e.g. "injection", "delivery", "bounce", "click", "open").
    #[serde(rename = "type")]
    pub event_type: String,
    /// Timestamp of the event (ISO 8601).
    pub timestamp: String,

    // ── Common optional fields ──
    /// Transmission request ID.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Recipient email address.
    #[serde(default)]
    pub rcpt_to: Option<String>,
    /// Original recipient email address.
    #[serde(default)]
    pub raw_rcpt_to: Option<String>,
    /// Domain of the recipient.
    #[serde(default)]
    pub recipient_domain: Option<String>,
    /// Mailbox provider of the recipient.
    #[serde(default)]
    pub mailbox_provider: Option<String>,
    /// Region of the mailbox provider.
    #[serde(default)]
    pub mailbox_provider_region: Option<String>,
    /// SMTP message ID.
    #[serde(default)]
    pub message_id: Option<String>,
    /// Email subject line.
    #[serde(default)]
    pub subject: Option<String>,
    /// Friendly from address.
    #[serde(default)]
    pub friendly_from: Option<String>,
    /// The sending domain.
    #[serde(default)]
    pub sending_domain: Option<String>,
    /// IP address used to send the email.
    #[serde(default)]
    pub sending_ip: Option<String>,
    /// Whether click tracking was enabled.
    #[serde(default)]
    pub click_tracking: Option<bool>,
    /// Whether open tracking was enabled.
    #[serde(default)]
    pub open_tracking: Option<bool>,
    /// Whether this was a transactional message.
    #[serde(default)]
    pub transactional: Option<bool>,
    /// Message size in bytes.
    #[serde(default)]
    pub msg_size: Option<u64>,
    /// When the message was injected (ISO 8601).
    #[serde(default)]
    pub injection_time: Option<String>,
    /// Recipient metadata.
    #[serde(default)]
    pub rcpt_meta: Option<serde_json::Value>,
    /// Campaign identifier.
    #[serde(default)]
    pub campaign_id: Option<String>,
    /// Template identifier.
    #[serde(default)]
    pub template_id: Option<String>,
    /// Template version.
    #[serde(default)]
    pub template_version: Option<String>,
    /// IP pool used for sending.
    #[serde(default)]
    pub ip_pool: Option<String>,
    /// Envelope sender (MAIL FROM).
    #[serde(default)]
    pub msg_from: Option<String>,
    /// Recipient type.
    #[serde(default)]
    pub rcpt_type: Option<String>,
    /// Recipient tags.
    #[serde(default)]
    pub rcpt_tags: Option<Vec<String>>,
    /// Whether AMP was enabled.
    #[serde(default)]
    pub amp_enabled: Option<bool>,
    /// Delivery method (e.g. "esmtp").
    #[serde(default)]
    pub delv_method: Option<String>,
    /// Reception method (e.g. "rest").
    #[serde(default)]
    pub recv_method: Option<String>,
    /// Routing domain.
    #[serde(default)]
    pub routing_domain: Option<String>,
    /// Scheduled delivery time.
    #[serde(default)]
    pub scheduled_time: Option<String>,
    /// A/B test identifier.
    #[serde(default)]
    pub ab_test_id: Option<String>,
    /// A/B test version.
    #[serde(default)]
    pub ab_test_version: Option<String>,

    // ── Bounce/delay/out_of_band/policy_rejection fields ──
    /// Bounce classification code.
    #[serde(default)]
    pub bounce_class: Option<i64>,
    /// SMTP error code.
    #[serde(default)]
    pub error_code: Option<String>,
    /// Human-readable bounce/delay/failure reason.
    #[serde(default)]
    pub reason: Option<String>,
    /// Raw SMTP reason string.
    #[serde(default)]
    pub raw_reason: Option<String>,
    /// Number of delivery retries attempted.
    #[serde(default)]
    pub num_retries: Option<u32>,
    /// Device token if applicable.
    #[serde(default)]
    pub device_token: Option<String>,

    // ── Delivery/delay fields ──
    /// Time spent in queue in milliseconds.
    #[serde(default)]
    pub queue_time: Option<u64>,
    /// Whether TLS was used for outbound delivery.
    #[serde(default)]
    pub outbound_tls: Option<String>,

    // ── Click fields ──
    /// The URL that was clicked.
    #[serde(default)]
    pub target_link_url: Option<String>,
    /// The name/label of the clicked link.
    #[serde(default)]
    pub target_link_name: Option<String>,

    // ── Open/click shared fields ──
    /// Raw user agent string.
    #[serde(default)]
    pub user_agent: Option<String>,
    /// Parsed user agent information.
    #[serde(default)]
    pub user_agent_parsed: Option<UserAgentParsed>,
    /// Geolocation data.
    #[serde(default)]
    pub geo_ip: Option<GeoIp>,
    /// IP address of the open/click.
    #[serde(default)]
    pub ip_address: Option<String>,
    /// Whether initial open tracking pixel was used.
    #[serde(default)]
    pub initial_pixel: Option<bool>,

    // ── Spam complaint fields ──
    /// Feedback type (e.g. "abuse").
    #[serde(default)]
    pub fbtype: Option<String>,
    /// Who reported the spam.
    #[serde(default)]
    pub report_by: Option<String>,
    /// Where the spam report was sent.
    #[serde(default)]
    pub report_to: Option<String>,

    // ── Policy rejection fields ──
    /// Remote IP address.
    #[serde(default)]
    pub remote_addr: Option<String>,
}

/// Parsed user agent information from open/click events.
#[derive(Debug, Clone, Deserialize)]
pub struct UserAgentParsed {
    /// Browser or email client family.
    #[serde(default)]
    pub agent_family: Option<String>,
    /// Device brand (e.g. Apple, Samsung).
    #[serde(default)]
    pub device_brand: Option<String>,
    /// Device family (e.g. iPhone, Desktop).
    #[serde(default)]
    pub device_family: Option<String>,
    /// Operating system family.
    #[serde(default)]
    pub os_family: Option<String>,
    /// Operating system version.
    #[serde(default)]
    pub os_version: Option<String>,
    /// Whether the device is mobile.
    #[serde(default)]
    pub is_mobile: Option<bool>,
    /// Whether the request came through a proxy.
    #[serde(default)]
    pub is_proxy: Option<bool>,
    /// Whether the open was prefetched by an email provider.
    #[serde(default)]
    pub is_prefetched: Option<bool>,
}

/// Geolocation data from open/click events.
#[derive(Debug, Clone, Deserialize)]
pub struct GeoIp {
    /// ISO 3166-1 alpha-2 country code.
    #[serde(default)]
    pub country: Option<String>,
    /// Region or state code.
    #[serde(default)]
    pub region: Option<String>,
    /// City name.
    #[serde(default)]
    pub city: Option<String>,
    /// Latitude.
    #[serde(default)]
    pub latitude: Option<f64>,
    /// Longitude.
    #[serde(default)]
    pub longitude: Option<f64>,
    /// ZIP code.
    #[serde(default)]
    pub zip: Option<String>,
    /// Postal code.
    #[serde(default)]
    pub postal_code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScheduledTransmissionResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ScheduledTransmission,
}

/// A scheduled email transmission.
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduledTransmission {
    /// Unique transmission ID.
    pub transmission_id: String,
    /// Current state of the transmission.
    pub state: String,
    /// Scheduled delivery time (ISO 8601).
    #[serde(default)]
    pub scheduled_at: Option<String>,
    /// Sender email address.
    pub from: String,
    /// Sender display name.
    #[serde(default)]
    pub from_name: Option<String>,
    /// Email subject line.
    pub subject: String,
    /// List of recipient email addresses.
    pub recipients: Vec<String>,
    /// Total number of recipients.
    pub num_recipients: u32,
    /// Delivery events for this transmission.
    pub events: Vec<EmailEvent>,
}
