use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::audience::AudiencePagination;
use crate::config::Config;

// ── Enum Types ────────────────────────────────────────────────────────────

/// Default subscription mode for a topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AudienceTopicDefaultSubscription {
    OptIn,
    OptOut,
    /// An unknown value not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for AudienceTopicDefaultSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OptIn => write!(f, "opt_in"),
            Self::OptOut => write!(f, "opt_out"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Visibility of a topic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AudienceTopicVisibility {
    Private,
    Public,
    /// An unknown value not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for AudienceTopicVisibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Private => write!(f, "private"),
            Self::Public => write!(f, "public"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Service for the `/audience/topics` endpoints.
#[derive(Clone, Debug)]
pub struct AudienceTopicsSvc(pub(crate) Arc<Config>);

impl AudienceTopicsSvc {
    /// List audience topics.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::audience::topics::ListAudienceTopicsOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    /// let response = client.audience.topics.list(ListAudienceTopicsOptions::new()).await?;
    /// for topic in &response.topics {
    ///     println!("{}: {}", topic.id, topic.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list(
        &self,
        options: ListAudienceTopicsOptions,
    ) -> crate::Result<ListAudienceTopicsResponse> {
        let mut request = self.0.build(Method::GET, "/audience/topics");

        if let Some(page) = options.page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListAudienceTopicsResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Create a new audience topic.
    #[maybe_async::maybe_async]
    pub async fn create(
        &self,
        options: CreateAudienceTopicOptions,
    ) -> crate::Result<AudienceTopic> {
        let request = self.0.build(Method::POST, "/audience/topics").json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowAudienceTopicResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Retrieve a single audience topic.
    #[maybe_async::maybe_async]
    pub async fn get(&self, topic_id: &str) -> crate::Result<AudienceTopic> {
        let path = format!("/audience/topics/{}", Config::encode_path_segment(topic_id));
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowAudienceTopicResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Update an audience topic. Note: `default_subscription` is immutable.
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        topic_id: &str,
        options: UpdateAudienceTopicOptions,
    ) -> crate::Result<AudienceTopic> {
        let path = format!("/audience/topics/{}", Config::encode_path_segment(topic_id));
        let request = self.0.build(Method::PATCH, &path).json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowAudienceTopicResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Delete an audience topic.
    #[maybe_async::maybe_async]
    pub async fn delete(&self, topic_id: &str) -> crate::Result<()> {
        let path = format!("/audience/topics/{}", Config::encode_path_segment(topic_id));
        let request = self.0.build(Method::DELETE, &path);
        self.0.send(request).await?;
        Ok(())
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for listing audience topics.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListAudienceTopicsOptions {
    page: Option<u32>,
    per_page: Option<u32>,
}

impl ListAudienceTopicsOptions {
    /// Creates new [`ListAudienceTopicsOptions`] with default values.
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
}

/// Options for creating an audience topic.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct CreateAudienceTopicOptions {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_subscription: Option<AudienceTopicDefaultSubscription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<AudienceTopicVisibility>,
}

impl CreateAudienceTopicOptions {
    /// Creates new [`CreateAudienceTopicOptions`] with the required name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            default_subscription: None,
            visibility: None,
        }
    }

    /// Sets the topic description.
    #[inline]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the default subscription mode (defaults to `opt_in` on the server).
    #[inline]
    pub fn with_default_subscription(
        mut self,
        default_subscription: AudienceTopicDefaultSubscription,
    ) -> Self {
        self.default_subscription = Some(default_subscription);
        self
    }

    /// Sets the topic visibility (defaults to `private` on the server).
    #[inline]
    pub fn with_visibility(mut self, visibility: AudienceTopicVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }
}

/// Options for updating an audience topic.
#[must_use]
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateAudienceTopicOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<AudienceTopicVisibility>,
}

impl UpdateAudienceTopicOptions {
    /// Creates new [`UpdateAudienceTopicOptions`] with no fields set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the new name.
    #[inline]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the new description.
    #[inline]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(Some(description.into()));
        self
    }

    /// Clears the topic description by sending `null`.
    #[inline]
    pub fn clear_description(mut self) -> Self {
        self.description = Some(None);
        self
    }

    /// Sets the new visibility.
    #[inline]
    pub fn with_visibility(mut self, visibility: AudienceTopicVisibility) -> Self {
        self.visibility = Some(visibility);
        self
    }
}

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListAudienceTopicsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListAudienceTopicsResponse,
}

/// Response from listing audience topics.
#[derive(Debug, Clone, Deserialize)]
pub struct ListAudienceTopicsResponse {
    /// The audience topics.
    pub topics: Vec<AudienceTopic>,
    /// Pagination metadata.
    pub pagination: AudiencePagination,
}

#[derive(Debug, Deserialize)]
struct ShowAudienceTopicResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: AudienceTopic,
}

/// An audience topic.
#[derive(Debug, Clone, Deserialize)]
pub struct AudienceTopic {
    /// Unique topic ID.
    pub id: String,
    /// Topic name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Default subscription mode.
    pub default_subscription: AudienceTopicDefaultSubscription,
    /// Visibility.
    pub visibility: AudienceTopicVisibility,
    /// Number of contacts subscribed to this topic.
    pub contacts_count: u64,
    /// Creation timestamp (may be `null` for topics created before timestamping was added).
    pub created_at: Option<String>,
}
