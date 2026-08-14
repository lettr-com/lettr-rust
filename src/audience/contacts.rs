use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use reqwest::Method;
use serde::de::{Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};

use crate::audience::AudiencePagination;
use crate::config::Config;

// ── Enum Types ────────────────────────────────────────────────────────────

/// Status of an audience contact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AudienceContactStatus {
    Subscribed,
    Unsubscribed,
    Bounced,
    Complained,
    Unverified,
    /// An unknown status not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for AudienceContactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subscribed => write!(f, "subscribed"),
            Self::Unsubscribed => write!(f, "unsubscribed"),
            Self::Bounced => write!(f, "bounced"),
            Self::Complained => write!(f, "complained"),
            Self::Unverified => write!(f, "unverified"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Status accepted by the contact update endpoint (only subscribed/unsubscribed are mutable).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum UpdateAudienceContactStatus {
    Subscribed,
    Unsubscribed,
}

/// What a write request should *do* with a topic.
///
/// Distinct from a topic's `default_subscription`, which describes how the
/// topic behaves for a contact that says nothing. [`OptOut`] here also cancels
/// the auto-subscription a topic whose default is `opt_out` would otherwise
/// give a newly created contact, so a create and an unsubscribe fit in one
/// request.
///
/// [`OptOut`]: AudienceTopicSubscriptionState::OptOut
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AudienceTopicSubscriptionState {
    OptIn,
    OptOut,
}

/// Reason a single row was skipped during a bulk contact create.
///
/// These are per-row codes reported inside a `201` body — not the top-level
/// [`ErrorCode`](crate::ErrorCode) of a failed request.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BulkAudienceContactErrorCode {
    MissingEmail,
    InvalidEmail,
    InvalidPropertyValue,
    UnknownPropertyKey,
    UnknownList,
    UnknownTopic,
    InvalidTopicSubscription,
    /// A code added server-side that this SDK version does not know.
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for BulkAudienceContactErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEmail => write!(f, "missing_email"),
            Self::InvalidEmail => write!(f, "invalid_email"),
            Self::InvalidPropertyValue => write!(f, "invalid_property_value"),
            Self::UnknownPropertyKey => write!(f, "unknown_property_key"),
            Self::UnknownList => write!(f, "unknown_list"),
            Self::UnknownTopic => write!(f, "unknown_topic"),
            Self::InvalidTopicSubscription => write!(f, "invalid_topic_subscription"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Service for the `/audience/contacts` endpoints (including list and topic memberships).
#[derive(Clone, Debug)]
pub struct AudienceContactsSvc(pub(crate) Arc<Config>);

impl AudienceContactsSvc {
    /// List audience contacts with optional filtering.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::audience::contacts::{ListAudienceContactsOptions, AudienceContactStatus};
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    /// let options = ListAudienceContactsOptions::new()
    ///     .status(AudienceContactStatus::Subscribed)
    ///     .per_page(50);
    /// let response = client.audience.contacts.list(options).await?;
    /// for contact in &response.contacts {
    ///     println!("{}: {}", contact.id, contact.email);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list(
        &self,
        options: ListAudienceContactsOptions,
    ) -> crate::Result<ListAudienceContactsResponse> {
        let mut request = self.0.build(Method::GET, "/audience/contacts");

        if let Some(page) = options.page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }
        if let Some(search) = options.search {
            request = request.query(&[("search", search)]);
        }
        if let Some(status) = options.status {
            request = request.query(&[("status", status.to_string())]);
        }
        if let Some(list_id) = options.list_id {
            request = request.query(&[("list_id", list_id)]);
        }
        if let Some(segment_id) = options.segment_id {
            request = request.query(&[("segment_id", segment_id)]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ListAudienceContactsResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Create a new audience contact.
    ///
    /// An email already in the team's audience comes back as
    /// `Error::Api` with [`ErrorCode::ResourceAlreadyExists`] (HTTP 409) — see
    /// [`Error::is_contact_already_exists`]. That is a client-correctable
    /// condition, not an outage: **do not retry it.** Use [`update`] instead,
    /// or [`bulk_create`] with `with_update_existing(true)`.
    ///
    /// [`ErrorCode::ResourceAlreadyExists`]: crate::ErrorCode::ResourceAlreadyExists
    /// [`Error::is_contact_already_exists`]: crate::Error::is_contact_already_exists
    /// [`update`]: Self::update
    /// [`bulk_create`]: Self::bulk_create
    #[maybe_async::maybe_async]
    pub async fn create(
        &self,
        options: CreateAudienceContactOptions,
    ) -> crate::Result<AudienceContact> {
        let request = self
            .0
            .build(Method::POST, "/audience/contacts")
            .json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ShowAudienceContactResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Bulk-create audience contacts (1–1000 rows per request).
    ///
    /// Rows that fail validation are skipped, not fatal: the call still returns
    /// HTTP 201 and reports them in the response's `errors`. An `Ok` result does
    /// not mean every row landed — check
    /// [`BulkCreateAudienceContactsResponse::has_errors`].
    #[maybe_async::maybe_async]
    pub async fn bulk_create(
        &self,
        options: BulkCreateAudienceContactsOptions,
    ) -> crate::Result<BulkCreateAudienceContactsResponse> {
        let request = self
            .0
            .build(Method::POST, "/audience/contacts/bulk")
            .json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<BulkCreateAudienceContactsResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Retrieve a single audience contact.
    #[maybe_async::maybe_async]
    pub async fn get(&self, contact_id: &str) -> crate::Result<AudienceContact> {
        let path = format!(
            "/audience/contacts/{}",
            Config::encode_path_segment(contact_id)
        );
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ShowAudienceContactResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Update an audience contact.
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        contact_id: &str,
        options: UpdateAudienceContactOptions,
    ) -> crate::Result<AudienceContact> {
        let path = format!(
            "/audience/contacts/{}",
            Config::encode_path_segment(contact_id)
        );
        let request = self.0.build(Method::PATCH, &path).json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ShowAudienceContactResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Delete an audience contact.
    #[maybe_async::maybe_async]
    pub async fn delete(&self, contact_id: &str) -> crate::Result<()> {
        let path = format!(
            "/audience/contacts/{}",
            Config::encode_path_segment(contact_id)
        );
        let request = self.0.build(Method::DELETE, &path);
        self.0.send(request).await?;
        Ok(())
    }

    /// Attach a contact to a list.
    #[maybe_async::maybe_async]
    pub async fn attach_to_list(&self, contact_id: &str, list_id: &str) -> crate::Result<()> {
        let path = format!(
            "/audience/contacts/{}/lists/{}",
            Config::encode_path_segment(contact_id),
            Config::encode_path_segment(list_id)
        );
        let request = self.0.build(Method::POST, &path);
        self.0.send(request).await?;
        Ok(())
    }

    /// Detach a contact from a list (idempotent).
    #[maybe_async::maybe_async]
    pub async fn detach_from_list(&self, contact_id: &str, list_id: &str) -> crate::Result<()> {
        let path = format!(
            "/audience/contacts/{}/lists/{}",
            Config::encode_path_segment(contact_id),
            Config::encode_path_segment(list_id)
        );
        let request = self.0.build(Method::DELETE, &path);
        self.0.send(request).await?;
        Ok(())
    }

    /// Bulk-attach contacts to lists (creates the Cartesian product of pairs).
    #[maybe_async::maybe_async]
    pub async fn bulk_attach_to_lists(
        &self,
        options: BulkContactListMembershipOptions,
    ) -> crate::Result<BulkAttachContactsToListsResponse> {
        let request = self
            .0
            .build(Method::POST, "/audience/contacts/lists/bulk")
            .json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<BulkAttachContactsToListsResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Bulk-detach contacts from lists.
    #[maybe_async::maybe_async]
    pub async fn bulk_detach_from_lists(
        &self,
        options: BulkContactListMembershipOptions,
    ) -> crate::Result<BulkDetachContactsFromListsResponse> {
        let request = self
            .0
            .build(Method::DELETE, "/audience/contacts/lists/bulk")
            .json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<BulkDetachContactsFromListsResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Subscribe a contact to a topic.
    #[maybe_async::maybe_async]
    pub async fn subscribe_to_topic(&self, contact_id: &str, topic_id: &str) -> crate::Result<()> {
        let path = format!(
            "/audience/contacts/{}/topics/{}",
            Config::encode_path_segment(contact_id),
            Config::encode_path_segment(topic_id)
        );
        let request = self.0.build(Method::POST, &path);
        self.0.send(request).await?;
        Ok(())
    }

    /// Unsubscribe a contact from a topic (idempotent).
    #[maybe_async::maybe_async]
    pub async fn unsubscribe_from_topic(
        &self,
        contact_id: &str,
        topic_id: &str,
    ) -> crate::Result<()> {
        let path = format!(
            "/audience/contacts/{}/topics/{}",
            Config::encode_path_segment(contact_id),
            Config::encode_path_segment(topic_id)
        );
        let request = self.0.build(Method::DELETE, &path);
        self.0.send(request).await?;
        Ok(())
    }

    /// Bulk-subscribe contacts to topics (the Cartesian product of
    /// `contact_ids` × `topic_ids`, up to 1000 × 50).
    ///
    /// Feed it [`BulkCreateAudienceContactsResponse::contact_ids`] from a bulk
    /// create — no ID lookup needed.
    #[maybe_async::maybe_async]
    pub async fn bulk_subscribe_to_topics(
        &self,
        options: BulkContactTopicMembershipOptions,
    ) -> crate::Result<BulkSubscribeContactsToTopicsResponse> {
        let request = self
            .0
            .build(Method::POST, "/audience/contacts/topics/bulk")
            .json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<BulkSubscribeContactsToTopicsResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Bulk-unsubscribe contacts from topics. Pairs that do not exist are
    /// ignored.
    ///
    /// This is a `DELETE` carrying a request body, as
    /// [`bulk_detach_from_lists`](Self::bulk_detach_from_lists) already is.
    #[maybe_async::maybe_async]
    pub async fn bulk_unsubscribe_from_topics(
        &self,
        options: BulkContactTopicMembershipOptions,
    ) -> crate::Result<BulkUnsubscribeContactsFromTopicsResponse> {
        let request = self
            .0
            .build(Method::DELETE, "/audience/contacts/topics/bulk")
            .json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<BulkUnsubscribeContactsFromTopicsResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for listing audience contacts.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListAudienceContactsOptions {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    status: Option<AudienceContactStatus>,
    list_id: Option<String>,
    segment_id: Option<String>,
}

impl ListAudienceContactsOptions {
    /// Creates new [`ListAudienceContactsOptions`] with default values.
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

    /// Sets a free-text search filter (matches email and properties).
    #[inline]
    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    /// Filters by contact status.
    #[inline]
    pub fn status(mut self, status: AudienceContactStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filters by list membership.
    #[inline]
    pub fn list_id(mut self, list_id: impl Into<String>) -> Self {
        self.list_id = Some(list_id.into());
        self
    }

    /// Filters by segment membership.
    #[inline]
    pub fn segment_id(mut self, segment_id: impl Into<String>) -> Self {
        self.segment_id = Some(segment_id.into());
        self
    }
}

/// Double opt-in confirmation email configuration.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct DoubleOptInConfig {
    /// `From` email address for the confirmation message.
    pub from: String,
    /// Optional `From` display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_name: Option<String>,
    /// Email subject line.
    pub subject: String,
    /// Slug of the template used for the confirmation email.
    pub template_slug: String,
    /// URL the recipient is sent to after confirming.
    pub redirect_url: String,
}

impl DoubleOptInConfig {
    /// Creates a new double opt-in configuration with the required fields.
    pub fn new(
        from: impl Into<String>,
        subject: impl Into<String>,
        template_slug: impl Into<String>,
        redirect_url: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            from_name: None,
            subject: subject.into(),
            template_slug: template_slug.into(),
            redirect_url: redirect_url.into(),
        }
    }

    /// Sets the optional `From` display name.
    #[inline]
    pub fn with_from_name(mut self, from_name: impl Into<String>) -> Self {
        self.from_name = Some(from_name.into());
        self
    }
}

/// Options for creating an audience contact.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct CreateAudienceContactOptions {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    list_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    double_opt_in: Option<DoubleOptInConfig>,
}

impl CreateAudienceContactOptions {
    /// Creates new [`CreateAudienceContactOptions`] with the required email.
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            list_id: None,
            properties: None,
            double_opt_in: None,
        }
    }

    /// Attaches the new contact to a list.
    #[inline]
    pub fn with_list_id(mut self, list_id: impl Into<String>) -> Self {
        self.list_id = Some(list_id.into());
        self
    }

    /// Sets custom properties on the new contact.
    #[inline]
    pub fn with_properties(mut self, properties: HashMap<String, String>) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Enables double opt-in: sends a confirmation email and marks the contact as `unverified`.
    #[inline]
    pub fn with_double_opt_in(mut self, double_opt_in: DoubleOptInConfig) -> Self {
        self.double_opt_in = Some(double_opt_in);
        self
    }
}

/// A topic and the subscription state to apply to it.
///
/// Used batch-wide on [`BulkCreateAudienceContactsOptions`] and per row on
/// [`BulkAudienceContactRow`]. A row-level opt-out wins over a batch-level
/// opt-in for that contact.
///
/// ```rust
/// # use lettr::audience::contacts::AudienceTopicSubscription;
/// let subscribe = AudienceTopicSubscription::opt_in("01h-newsletter");
/// let suppress = AudienceTopicSubscription::opt_out("01h-promos");
/// # let _ = (subscribe, suppress);
/// ```
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct AudienceTopicSubscription {
    /// Topic ID.
    pub id: String,
    /// What to do with the topic.
    pub subscription: AudienceTopicSubscriptionState,
}

impl AudienceTopicSubscription {
    /// Subscribes the contact to the topic.
    pub fn opt_in(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            subscription: AudienceTopicSubscriptionState::OptIn,
        }
    }

    /// Suppresses the topic for the contact — including a topic that would
    /// otherwise auto-subscribe newly created contacts.
    pub fn opt_out(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            subscription: AudienceTopicSubscriptionState::OptOut,
        }
    }
}

/// One contact in a bulk-create payload.
///
/// `list_ids` and `topics` here are applied **on top of** the batch-wide ones on
/// [`BulkCreateAudienceContactsOptions`]; a property key here overrides the
/// batch-wide value for the same key.
///
/// A row that fails validation is skipped rather than failing the request — it
/// comes back in [`BulkCreateAudienceContactsResponse::errors`].
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct BulkAudienceContactRow {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    list_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topics: Option<Vec<AudienceTopicSubscription>>,
}

impl BulkAudienceContactRow {
    /// Creates a row for the given address. It inherits everything batch-wide
    /// until the `with_*` methods add row-level values.
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            properties: None,
            list_ids: None,
            topics: None,
        }
    }

    /// Sets property values for this contact. Each key must match a property
    /// defined for the team, and wins over the batch-wide value.
    #[inline]
    pub fn with_properties(mut self, properties: HashMap<String, String>) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Sets up to 50 lists for this row, on top of the batch-wide ones.
    #[inline]
    pub fn with_list_ids(mut self, list_ids: Vec<String>) -> Self {
        self.list_ids = Some(list_ids);
        self
    }

    /// Sets up to 50 topic subscriptions for this row.
    #[inline]
    pub fn with_topics(mut self, topics: Vec<AudienceTopicSubscription>) -> Self {
        self.topics = Some(topics);
        self
    }
}

/// Options for bulk-creating audience contacts.
///
/// Two shapes are supported, and exactly one of them must be filled in:
///
/// - [`new`](Self::new) — a flat list of addresses that all share the batch-wide
///   list, properties and topics. The original shape, unchanged.
/// - [`for_contacts`](Self::for_contacts) — one [`BulkAudienceContactRow`] per
///   contact, each with its own properties, lists and topic subscriptions.
///
/// Batch-wide `list_ids` and `topics` are unioned into every row; a row-level
/// property key or opt-out wins over the batch-wide value.
///
/// ```rust
/// # use lettr::audience::contacts::{
/// #     AudienceTopicSubscription, BulkAudienceContactRow, BulkCreateAudienceContactsOptions,
/// # };
/// let options = BulkCreateAudienceContactsOptions::for_contacts(vec![
///     BulkAudienceContactRow::new("cara@example.com"),
///     BulkAudienceContactRow::new("dan@example.com")
///         .with_topics(vec![AudienceTopicSubscription::opt_out("01h-promos")]),
/// ])
/// .with_list_ids(vec!["01h-everyone".into()])
/// .with_update_existing(true);
/// # let _ = options;
/// ```
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct BulkCreateAudienceContactsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    emails: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    list_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contacts: Option<Vec<BulkAudienceContactRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    list_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topics: Option<Vec<AudienceTopicSubscription>>,
    // Skipped when false so a legacy payload stays byte-identical; the API
    // defaults it to false anyway.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    update_existing: bool,
}

impl BulkCreateAudienceContactsOptions {
    /// Creates new options from a vector of email addresses (1–1000 items).
    pub fn new(emails: Vec<String>) -> Self {
        Self {
            emails: Some(emails),
            list_id: None,
            properties: None,
            contacts: None,
            list_ids: None,
            topics: None,
            update_existing: false,
        }
    }

    /// Creates new options from per-contact rows (1–1000 items), each with its
    /// own properties, lists and topic subscriptions.
    pub fn for_contacts(contacts: Vec<BulkAudienceContactRow>) -> Self {
        Self {
            emails: None,
            list_id: None,
            properties: None,
            contacts: Some(contacts),
            list_ids: None,
            topics: None,
            update_existing: false,
        }
    }

    /// Attaches all created contacts to a list. Folded into `list_ids` server-side.
    #[inline]
    pub fn with_list_id(mut self, list_id: impl Into<String>) -> Self {
        self.list_id = Some(list_id.into());
        self
    }

    /// Sets shared properties applied to every contact in the batch. A row's own
    /// key wins over these.
    #[inline]
    pub fn with_properties(mut self, properties: HashMap<String, String>) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Sets up to 50 batch-wide lists, unioned into every row.
    #[inline]
    pub fn with_list_ids(mut self, list_ids: Vec<String>) -> Self {
        self.list_ids = Some(list_ids);
        self
    }

    /// Sets up to 50 batch-wide topic subscriptions.
    #[inline]
    pub fn with_topics(mut self, topics: Vec<AudienceTopicSubscription>) -> Self {
        self.topics = Some(topics);
        self
    }

    /// When `true`, existing contacts have their properties merged (submitted
    /// keys overwrite, absent keys are preserved) and opt-outs applied.
    ///
    /// Defaults to `false`, in which case existing contacts keep their
    /// properties but are still attached to the requested lists.
    #[inline]
    pub fn with_update_existing(mut self, update_existing: bool) -> Self {
        self.update_existing = update_existing;
        self
    }
}

/// Options for updating an audience contact.
///
/// To clear a property, set its value to `None` in the `properties` map — this
/// serializes as `null` and the server removes the property.
#[must_use]
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateAudienceContactOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<UpdateAudienceContactStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, Option<String>>>,
}

impl UpdateAudienceContactOptions {
    /// Creates new [`UpdateAudienceContactOptions`] with no fields set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the new email address.
    #[inline]
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the new status (only `subscribed`/`unsubscribed` are accepted).
    #[inline]
    pub fn with_status(mut self, status: UpdateAudienceContactStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets properties. Values of `None` clear the corresponding property.
    #[inline]
    pub fn with_properties(mut self, properties: HashMap<String, Option<String>>) -> Self {
        self.properties = Some(properties);
        self
    }
}

/// Options for the bulk attach/detach contacts-to-lists endpoints.
///
/// Use the named builder methods to avoid mixing up the two ID lists:
///
/// ```rust,no_run
/// # use lettr::audience::contacts::BulkContactListMembershipOptions;
/// let opts = BulkContactListMembershipOptions::new()
///     .with_contact_ids(vec!["contact-1".into(), "contact-2".into()])
///     .with_list_ids(vec!["list-1".into()]);
/// # let _ = opts;
/// ```
#[must_use]
#[derive(Debug, Default, Clone, Serialize)]
pub struct BulkContactListMembershipOptions {
    contact_ids: Vec<String>,
    list_ids: Vec<String>,
}

impl BulkContactListMembershipOptions {
    /// Creates empty options. The server requires both ID lists to be non-empty;
    /// add them via [`Self::with_contact_ids`] and [`Self::with_list_ids`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the contact IDs (1–1000 items per request).
    #[inline]
    pub fn with_contact_ids(mut self, contact_ids: Vec<String>) -> Self {
        self.contact_ids = contact_ids;
        self
    }

    /// Sets the list IDs (1–50 items per request).
    #[inline]
    pub fn with_list_ids(mut self, list_ids: Vec<String>) -> Self {
        self.list_ids = list_ids;
        self
    }
}

/// Options for the bulk subscribe/unsubscribe contacts-to-topics endpoints.
///
/// Both directions take the same body and process the Cartesian product of
/// `contact_ids` × `topic_ids`.
///
/// ```rust,no_run
/// # use lettr::audience::contacts::BulkContactTopicMembershipOptions;
/// let opts = BulkContactTopicMembershipOptions::new()
///     .with_contact_ids(vec!["contact-1".into(), "contact-2".into()])
///     .with_topic_ids(vec!["topic-1".into()]);
/// # let _ = opts;
/// ```
#[must_use]
#[derive(Debug, Default, Clone, Serialize)]
pub struct BulkContactTopicMembershipOptions {
    contact_ids: Vec<String>,
    topic_ids: Vec<String>,
}

impl BulkContactTopicMembershipOptions {
    /// Creates empty options. The server requires both ID lists to be non-empty;
    /// add them via [`Self::with_contact_ids`] and [`Self::with_topic_ids`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the contact IDs (1–1000 items per request).
    #[inline]
    pub fn with_contact_ids(mut self, contact_ids: Vec<String>) -> Self {
        self.contact_ids = contact_ids;
        self
    }

    /// Sets the topic IDs (1–50 items per request).
    #[inline]
    pub fn with_topic_ids(mut self, topic_ids: Vec<String>) -> Self {
        self.topic_ids = topic_ids;
        self
    }
}

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListAudienceContactsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListAudienceContactsResponse,
}

/// Response from listing audience contacts.
#[derive(Debug, Clone, Deserialize)]
pub struct ListAudienceContactsResponse {
    /// The audience contacts.
    pub contacts: Vec<AudienceContact>,
    /// Pagination metadata.
    pub pagination: AudiencePagination,
}

#[derive(Debug, Deserialize)]
struct ShowAudienceContactResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: AudienceContact,
}

/// An audience contact.
#[derive(Debug, Clone, Deserialize)]
pub struct AudienceContact {
    /// Unique contact ID.
    pub id: String,
    /// Email address.
    pub email: String,
    /// Subscription status.
    pub status: AudienceContactStatus,
    /// Custom properties.
    #[serde(deserialize_with = "deserialize_string_map_or_empty_seq")]
    pub properties: HashMap<String, String>,
    /// Creation timestamp.
    pub created_at: String,
    /// Lists this contact belongs to.
    pub lists: Vec<AudienceContactListLink>,
    /// Topics this contact is subscribed to.
    pub topics: Vec<AudienceContactTopicLink>,
}

/// Minimal reference to a list from a contact view.
#[derive(Debug, Clone, Deserialize)]
pub struct AudienceContactListLink {
    /// List ID.
    pub id: String,
    /// List name.
    pub name: String,
}

/// Minimal reference to a topic from a contact view.
#[derive(Debug, Clone, Deserialize)]
pub struct AudienceContactTopicLink {
    /// Topic ID.
    pub id: String,
    /// Topic name.
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct BulkCreateAudienceContactsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: BulkCreateAudienceContactsResponse,
}

/// Response from the bulk contact creation endpoint.
///
/// A bulk create can **partially succeed**: rows that fail validation are
/// skipped and reported in [`errors`](Self::errors) while the rest of the batch
/// is written, and the call still returns HTTP 201. An `Ok` result therefore
/// does **not** mean every row landed — check [`has_errors`](Self::has_errors).
///
/// [`already_existed`](Self::already_existed) and [`updated`](Self::updated)
/// overlap by design. They answer different questions ("was the address already
/// in the audience?" vs "did this request change the contact?"), so the counters
/// do not sum to the row count: a contact that already existed and got attached
/// to a list is counted in both.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkCreateAudienceContactsResponse {
    /// Number of contacts newly created.
    pub created: u32,
    /// Number of emails skipped because they already existed.
    pub already_existed: u32,
    /// Existing contacts this request changed — properties merged, a list or
    /// topic attached, or a subscription dropped.
    #[serde(default)]
    pub updated: u32,
    /// Number of skipped rows.
    #[serde(default)]
    pub error_count: u32,
    /// The skipped rows.
    #[serde(default)]
    pub errors: Vec<BulkAudienceContactError>,
    /// Every contact that exists after the request, in submission order.
    #[serde(default)]
    pub contacts: Vec<BulkAudienceContactRef>,
}

impl BulkCreateAudienceContactsResponse {
    /// Whether any row was skipped. Always check this — a bulk create reports
    /// partial failures in the body, not in the HTTP status.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// The IDs of every contact that exists after the request, in submission
    /// order — ready to feed into the bulk list and topic endpoints.
    #[must_use]
    pub fn contact_ids(&self) -> Vec<String> {
        self.contacts.iter().map(|c| c.id.clone()).collect()
    }

    /// Looks up the ID for a submitted address. Matching is case-insensitive
    /// because the API normalizes addresses before storing them.
    #[must_use]
    pub fn id_for(&self, email: &str) -> Option<&str> {
        let needle = email.trim().to_lowercase();
        self.contacts
            .iter()
            .find(|c| c.email.to_lowercase() == needle)
            .map(|c| c.id.as_str())
    }
}

/// A row that was skipped during a bulk create.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkAudienceContactError {
    /// Zero-based position of the row in the submitted list.
    pub index: u32,
    /// The submitted address, when the row had one.
    pub email: Option<String>,
    /// Why the row was skipped.
    pub error_code: BulkAudienceContactErrorCode,
    /// Human-readable reason.
    pub error: String,
}

/// Identity of a contact that exists after a bulk create, so the caller can
/// chain into the bulk list and topic endpoints without looking IDs up again.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkAudienceContactRef {
    /// Contact ID.
    pub id: String,
    /// Normalized email address.
    pub email: String,
    /// `true` when this request created the contact, `false` when it already existed.
    pub created: bool,
}

#[derive(Debug, Deserialize)]
struct BulkAttachContactsToListsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: BulkAttachContactsToListsResponse,
}

/// Response from the bulk attach contacts-to-lists endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkAttachContactsToListsResponse {
    /// Number of new (contact, list) pairs attached.
    pub attached: u32,
    /// Number of pairs that were already attached.
    pub already_attached: u32,
    /// Total number of (contact, list) pairs processed.
    pub total_pairs: u32,
}

#[derive(Debug, Deserialize)]
struct BulkDetachContactsFromListsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: BulkDetachContactsFromListsResponse,
}

/// Response from the bulk detach contacts-from-lists endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkDetachContactsFromListsResponse {
    /// Number of (contact, list) pairs detached.
    pub detached: u32,
    /// Number of pairs that were not present.
    pub not_present: u32,
    /// Total number of (contact, list) pairs processed.
    pub total_pairs: u32,
}

#[derive(Debug, Deserialize)]
struct BulkSubscribeContactsToTopicsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: BulkSubscribeContactsToTopicsResponse,
}

/// Response from the bulk subscribe contacts-to-topics endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkSubscribeContactsToTopicsResponse {
    /// Number of new (contact, topic) pairs subscribed.
    pub subscribed: u32,
    /// Number of pairs that were already subscribed.
    pub already_subscribed: u32,
    /// Total number of (contact, topic) pairs processed.
    pub total_pairs: u32,
}

#[derive(Debug, Deserialize)]
struct BulkUnsubscribeContactsFromTopicsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: BulkUnsubscribeContactsFromTopicsResponse,
}

/// Response from the bulk unsubscribe contacts-from-topics endpoint.
///
/// Pairs that did not exist are ignored, so `unsubscribed` can be lower than
/// `total_pairs`.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkUnsubscribeContactsFromTopicsResponse {
    /// Number of (contact, topic) pairs unsubscribed.
    pub unsubscribed: u32,
    /// Total number of (contact, topic) pairs processed.
    pub total_pairs: u32,
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Deserialize a JSON object into a `HashMap<String, String>`, also accepting
/// an empty JSON array `[]` as an empty map.
///
/// The Lettr API is PHP-backed, and PHP's `json_encode` serializes an empty
/// associative array as `[]` rather than `{}`. Without this shim a contact
/// with no custom properties fails with `invalid type: sequence, expected a map`.
fn deserialize_string_map_or_empty_seq<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MapOrEmptySeq(PhantomData<HashMap<String, String>>);

    impl<'de> Visitor<'de> for MapOrEmptySeq {
        type Value = HashMap<String, String>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a JSON object of string→string, or an empty JSON array")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut out = HashMap::with_capacity(access.size_hint().unwrap_or(0));
            while let Some((key, value)) = access.next_entry::<String, String>()? {
                out.insert(key, value);
            }
            Ok(out)
        }

        fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // Only an *empty* sequence is acceptable — a non-empty array would
            // be a real shape mismatch we shouldn't silently swallow.
            if access.next_element::<IgnoredAny>()?.is_some() {
                return Err(serde::de::Error::custom(
                    "expected a map of string→string or an empty array, got a non-empty array",
                ));
            }
            Ok(HashMap::new())
        }
    }

    deserializer.deserialize_any(MapOrEmptySeq(PhantomData))
}
