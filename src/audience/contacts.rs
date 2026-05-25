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
        let wrapper = response.json::<ShowAudienceContactResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Bulk-create audience contacts (1–1000 emails per request).
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
        let wrapper = response.json::<ShowAudienceContactResponseWrapper>().await?;
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
        let wrapper = response.json::<ShowAudienceContactResponseWrapper>().await?;
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
    pub async fn subscribe_to_topic(
        &self,
        contact_id: &str,
        topic_id: &str,
    ) -> crate::Result<()> {
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

/// Options for bulk-creating audience contacts.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct BulkCreateAudienceContactsOptions {
    emails: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    list_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, String>>,
}

impl BulkCreateAudienceContactsOptions {
    /// Creates new options from a vector of email addresses (1–1000 items).
    pub fn new(emails: Vec<String>) -> Self {
        Self {
            emails,
            list_id: None,
            properties: None,
        }
    }

    /// Attaches all created contacts to a list.
    #[inline]
    pub fn with_list_id(mut self, list_id: impl Into<String>) -> Self {
        self.list_id = Some(list_id.into());
        self
    }

    /// Sets shared properties applied to every created contact.
    #[inline]
    pub fn with_properties(mut self, properties: HashMap<String, String>) -> Self {
        self.properties = Some(properties);
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
#[derive(Debug, Clone, Deserialize)]
pub struct BulkCreateAudienceContactsResponse {
    /// Number of contacts newly created.
    pub created: u32,
    /// Number of emails skipped because they already existed.
    pub already_existed: u32,
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
