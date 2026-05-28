//! Campaign management: listing, engagement stats and events, and dispatch
//! actions (send, schedule, unschedule).
//!
//! Access through the [`Lettr::campaigns`](crate::Lettr) field.

use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::config::Config;

// ── Enum Types ────────────────────────────────────────────────────────────

/// Lifecycle status of a campaign.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CampaignStatus {
    Draft,
    Scheduled,
    Preparing,
    InReview,
    Sending,
    Sent,
    Failed,
    /// An unknown status not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Scheduled => write!(f, "scheduled"),
            Self::Preparing => write!(f, "preparing"),
            Self::InReview => write!(f, "in_review"),
            Self::Sending => write!(f, "sending"),
            Self::Sent => write!(f, "sent"),
            Self::Failed => write!(f, "failed"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Type of a campaign engagement event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CampaignEventType {
    Injection,
    Delivery,
    Bounce,
    SpamComplaint,
    Open,
    Click,
    ListUnsubscribe,
    /// An unknown event type not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for CampaignEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Injection => write!(f, "injection"),
            Self::Delivery => write!(f, "delivery"),
            Self::Bounce => write!(f, "bounce"),
            Self::SpamComplaint => write!(f, "spam_complaint"),
            Self::Open => write!(f, "open"),
            Self::Click => write!(f, "click"),
            Self::ListUnsubscribe => write!(f, "list_unsubscribe"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Service for the `/campaigns` endpoints.
#[derive(Clone, Debug)]
pub struct CampaignsSvc(pub(crate) Arc<Config>);

impl CampaignsSvc {
    /// List campaigns with optional status filter and pagination.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::campaigns::{ListCampaignsOptions, CampaignStatus};
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let options = ListCampaignsOptions::new()
    ///     .status(CampaignStatus::Sent)
    ///     .per_page(50);
    /// let response = client.campaigns.list(options).await?;
    ///
    /// for campaign in &response.campaigns {
    ///     println!("{}: {} ({} sent)", campaign.id, campaign.name, campaign.sent_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list(
        &self,
        options: ListCampaignsOptions,
    ) -> crate::Result<ListCampaignsResponse> {
        let mut request = self.0.build(Method::GET, "/campaigns");

        if let Some(page) = options.page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }
        if let Some(status) = options.status {
            request = request.query(&[("status", status.to_string())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListCampaignsResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Retrieve a single campaign, including rendered HTML content.
    #[maybe_async::maybe_async]
    pub async fn get(&self, campaign_id: &str) -> crate::Result<CampaignDetail> {
        let path = format!("/campaigns/{}", Config::encode_path_segment(campaign_id));
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowCampaignResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// List engagement events for a campaign.
    ///
    /// Uses cursor-based pagination: keep calling with the returned
    /// [`ListCampaignEventsResponse::next_cursor`] until it is `None`. When a
    /// filter is applied, an empty `events` page with a non-`None`
    /// `next_cursor` is valid mid-stream — continue paginating.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::campaigns::{ListCampaignEventsOptions, CampaignEventType};
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let mut cursor: Option<String> = None;
    /// loop {
    ///     let mut options = ListCampaignEventsOptions::new()
    ///         .event_type(CampaignEventType::Open)
    ///         .limit(100);
    ///     if let Some(c) = cursor.take() {
    ///         options = options.cursor(c);
    ///     }
    ///     let page = client.campaigns.list_events("campaign-id", options).await?;
    ///     for event in &page.events {
    ///         println!("{} {}", event.timestamp, event.email);
    ///     }
    ///     match page.next_cursor {
    ///         Some(c) => cursor = Some(c),
    ///         None => break,
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list_events(
        &self,
        campaign_id: &str,
        options: ListCampaignEventsOptions,
    ) -> crate::Result<ListCampaignEventsResponse> {
        let path = format!(
            "/campaigns/{}/events",
            Config::encode_path_segment(campaign_id)
        );
        let mut request = self.0.build(Method::GET, &path);

        if let Some(event_type) = options.event_type {
            request = request.query(&[("event_type", event_type.to_string())]);
        }
        if let Some(email) = options.email {
            request = request.query(&[("email", email)]);
        }
        if let Some(start_date) = options.start_date {
            request = request.query(&[("start_date", start_date)]);
        }
        if let Some(end_date) = options.end_date {
            request = request.query(&[("end_date", end_date)]);
        }
        if let Some(limit) = options.limit {
            request = request.query(&[("limit", limit.to_string())]);
        }
        if let Some(cursor) = options.cursor {
            request = request.query(&[("cursor", cursor)]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListCampaignEventsResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Dispatch a draft campaign immediately.
    ///
    /// Returns the updated campaign on success. May return `None` in the rare
    /// case the campaign cannot be re-read immediately after the action (e.g.
    /// concurrent deletion). Not available to sandbox API keys.
    #[maybe_async::maybe_async]
    pub async fn send(&self, campaign_id: &str) -> crate::Result<Option<Campaign>> {
        let path = format!(
            "/campaigns/{}/send",
            Config::encode_path_segment(campaign_id)
        );
        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<CampaignActionResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Schedule a campaign for future delivery, or reschedule an
    /// already-scheduled campaign. Not available to sandbox API keys.
    #[maybe_async::maybe_async]
    pub async fn schedule(
        &self,
        campaign_id: &str,
        options: ScheduleCampaignOptions,
    ) -> crate::Result<Option<Campaign>> {
        let path = format!(
            "/campaigns/{}/schedule",
            Config::encode_path_segment(campaign_id)
        );
        let request = self.0.build(Method::POST, &path).json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<CampaignActionResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Cancel a scheduled send, returning the campaign to `draft`. Not
    /// available to sandbox API keys.
    #[maybe_async::maybe_async]
    pub async fn unschedule(&self, campaign_id: &str) -> crate::Result<Option<Campaign>> {
        let path = format!(
            "/campaigns/{}/unschedule",
            Config::encode_path_segment(campaign_id)
        );
        let request = self.0.build(Method::POST, &path);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<CampaignActionResponseWrapper>().await?;
        Ok(wrapper.data)
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for listing campaigns.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListCampaignsOptions {
    page: Option<u32>,
    per_page: Option<u32>,
    status: Option<CampaignStatus>,
}

impl ListCampaignsOptions {
    /// Creates new [`ListCampaignsOptions`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the page number.
    #[inline]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the number of results per page (1-100).
    #[inline]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Filters by campaign status.
    #[inline]
    pub fn status(mut self, status: CampaignStatus) -> Self {
        self.status = Some(status);
        self
    }
}

/// Options for listing campaign engagement events.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListCampaignEventsOptions {
    event_type: Option<CampaignEventType>,
    email: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
}

impl ListCampaignEventsOptions {
    /// Creates new [`ListCampaignEventsOptions`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters by event type.
    #[inline]
    pub fn event_type(mut self, event_type: CampaignEventType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    /// Filters by recipient email address.
    #[inline]
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Only return events at or after this time (ISO 8601). A date-only value
    /// like `2026-05-01` is treated as the start of that day in UTC.
    #[inline]
    pub fn start_date(mut self, start_date: impl Into<String>) -> Self {
        self.start_date = Some(start_date.into());
        self
    }

    /// Only return events at or before this time (ISO 8601), inclusive. A
    /// date-only value covers the whole of that day in UTC.
    #[inline]
    pub fn end_date(mut self, end_date: impl Into<String>) -> Self {
        self.end_date = Some(end_date.into());
        self
    }

    /// Maximum events per page (1-100, default 25).
    #[inline]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Opaque pagination cursor from a previous response.
    #[inline]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }
}

/// Request body for scheduling a campaign.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct ScheduleCampaignOptions {
    scheduled_at: String,
}

impl ScheduleCampaignOptions {
    /// Creates new [`ScheduleCampaignOptions`] with the required delivery
    /// time. The value must be a future ISO 8601 timestamp; include a
    /// timezone offset (e.g. `+02:00` or `Z`). A value without an offset is
    /// interpreted as UTC.
    pub fn new(scheduled_at: impl Into<String>) -> Self {
        Self {
            scheduled_at: scheduled_at.into(),
        }
    }
}

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListCampaignsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListCampaignsResponse,
}

/// Response from listing campaigns.
#[derive(Debug, Clone, Deserialize)]
pub struct ListCampaignsResponse {
    /// The campaigns.
    pub campaigns: Vec<Campaign>,
    /// Pagination metadata.
    pub pagination: CampaignPagination,
}

/// Pagination metadata for campaign list responses.
#[derive(Debug, Clone, Deserialize)]
pub struct CampaignPagination {
    /// Total number of campaigns.
    pub total: u64,
    /// Results per page.
    pub per_page: u32,
    /// Current page number.
    pub current_page: u32,
    /// Last page number.
    pub last_page: u32,
}

#[derive(Debug, Deserialize)]
struct ShowCampaignResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: CampaignDetail,
}

/// Campaign summary with embedded engagement stats. Returned by the list
/// endpoint and embedded in action responses.
#[derive(Debug, Clone, Deserialize)]
pub struct Campaign {
    /// Unique campaign ID.
    pub id: String,
    /// Campaign name.
    pub name: String,
    /// Email subject line.
    #[serde(default)]
    pub subject: Option<String>,
    /// Sender email address.
    #[serde(default)]
    pub from_email: Option<String>,
    /// Sender display name.
    #[serde(default)]
    pub from_name: Option<String>,
    /// Reply-to address.
    #[serde(default)]
    pub reply_to: Option<String>,
    /// Campaign status.
    pub status: CampaignStatus,
    /// Scheduled delivery time (ISO 8601).
    #[serde(default)]
    pub scheduled_at: Option<String>,
    /// Total recipients resolved for the send.
    #[serde(default)]
    pub total_recipients: Option<u64>,
    /// Number of recipients the send completed for.
    pub sent_count: u64,
    /// When the campaign finished sending (ISO 8601).
    #[serde(default)]
    pub sent_at: Option<String>,
    /// Creation timestamp.
    pub created_at: String,
    /// Embedded engagement stats.
    pub stats: CampaignStats,
}

/// Full campaign representation including rendered HTML content. Returned by
/// the show endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CampaignDetail {
    /// Unique campaign ID.
    pub id: String,
    /// Campaign name.
    pub name: String,
    /// Email subject line.
    #[serde(default)]
    pub subject: Option<String>,
    /// Sender email address.
    #[serde(default)]
    pub from_email: Option<String>,
    /// Sender display name.
    #[serde(default)]
    pub from_name: Option<String>,
    /// Reply-to address.
    #[serde(default)]
    pub reply_to: Option<String>,
    /// Campaign status.
    pub status: CampaignStatus,
    /// Scheduled delivery time (ISO 8601).
    #[serde(default)]
    pub scheduled_at: Option<String>,
    /// Total recipients resolved for the send.
    #[serde(default)]
    pub total_recipients: Option<u64>,
    /// Number of recipients the send completed for.
    pub sent_count: u64,
    /// When the campaign finished sending (ISO 8601).
    #[serde(default)]
    pub sent_at: Option<String>,
    /// Creation timestamp.
    pub created_at: String,
    /// Embedded engagement stats.
    pub stats: CampaignStats,
    /// Rendered HTML content of the campaign email.
    #[serde(default)]
    pub html_content: Option<String>,
}

/// Aggregated engagement statistics for a campaign.
#[derive(Debug, Clone, Deserialize)]
pub struct CampaignStats {
    /// Number of injections (messages accepted for delivery).
    pub injections: u64,
    /// Number of successful deliveries.
    pub deliveries: u64,
    /// Number of bounces.
    pub bounces: u64,
    /// Number of spam complaints.
    pub spam_complaints: u64,
    /// Total opens (including repeats).
    pub opens: u64,
    /// Unique recipients that opened.
    pub unique_opens: u64,
    /// Total clicks (including repeats).
    pub clicks: u64,
    /// Unique recipients that clicked.
    pub unique_clicks: u64,
    /// Number of unsubscribes.
    pub unsubscribes: u64,
}

#[derive(Debug, Deserialize)]
struct ListCampaignEventsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListCampaignEventsResponse,
}

/// Response from listing campaign engagement events.
#[derive(Debug, Clone, Deserialize)]
pub struct ListCampaignEventsResponse {
    /// The events on this page.
    pub events: Vec<CampaignEvent>,
    /// Opaque cursor for the next page; `None` when there are no more events.
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// A single campaign engagement event.
#[derive(Debug, Clone, Deserialize)]
pub struct CampaignEvent {
    /// Unique event ID.
    pub event_id: String,
    /// Type of event.
    pub event_type: CampaignEventType,
    /// Recipient email address.
    pub email: String,
    /// When the event occurred (ISO 8601).
    pub timestamp: String,
    /// SparkPost bounce classification code (bounce events).
    #[serde(default)]
    pub bounce_class: Option<String>,
    /// Failure or bounce reason.
    #[serde(default)]
    pub reason: Option<String>,
    /// Clicked link URL (click events only).
    #[serde(default)]
    pub target_link_url: Option<String>,
    /// Recipient user agent (open/click events).
    #[serde(default)]
    pub user_agent: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CampaignActionResponseWrapper {
    #[allow(dead_code)]
    message: String,
    /// The updated campaign. Optional because the server may omit it if the
    /// campaign cannot be re-read immediately after the action.
    #[serde(default)]
    data: Option<Campaign>,
}
