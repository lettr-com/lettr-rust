use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::audience::AudiencePagination;
use crate::config::Config;

// ── Enum Types ────────────────────────────────────────────────────────────

/// Operator used in a segment condition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SegmentOperator {
    Contains,
    NotContains,
    Equals,
    NotEquals,
    StartsWith,
    NotStartsWith,
    EndsWith,
    NotEndsWith,
    IsTrue,
    IsFalse,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Before,
    After,
    /// An unknown operator not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for SegmentOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Contains => write!(f, "contains"),
            Self::NotContains => write!(f, "not_contains"),
            Self::Equals => write!(f, "equals"),
            Self::NotEquals => write!(f, "not_equals"),
            Self::StartsWith => write!(f, "starts_with"),
            Self::NotStartsWith => write!(f, "not_starts_with"),
            Self::EndsWith => write!(f, "ends_with"),
            Self::NotEndsWith => write!(f, "not_ends_with"),
            Self::IsTrue => write!(f, "is_true"),
            Self::IsFalse => write!(f, "is_false"),
            Self::GreaterThan => write!(f, "greater_than"),
            Self::GreaterThanOrEqual => write!(f, "greater_than_or_equal"),
            Self::LessThan => write!(f, "less_than"),
            Self::LessThanOrEqual => write!(f, "less_than_or_equal"),
            Self::Before => write!(f, "before"),
            Self::After => write!(f, "after"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Service for the `/audience/segments` endpoints.
#[derive(Clone, Debug)]
pub struct AudienceSegmentsSvc(pub(crate) Arc<Config>);

impl AudienceSegmentsSvc {
    /// List audience segments.
    #[maybe_async::maybe_async]
    pub async fn list(
        &self,
        options: ListAudienceSegmentsOptions,
    ) -> crate::Result<ListAudienceSegmentsResponse> {
        let mut request = self.0.build(Method::GET, "/audience/segments");

        if let Some(page) = options.page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }
        if let Some(list_id) = options.list_id {
            request = request.query(&[("list_id", list_id)]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ListAudienceSegmentsResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Create a new audience segment.
    #[maybe_async::maybe_async]
    pub async fn create(
        &self,
        options: CreateAudienceSegmentOptions,
    ) -> crate::Result<AudienceSegment> {
        let request = self
            .0
            .build(Method::POST, "/audience/segments")
            .json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ShowAudienceSegmentResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Retrieve a single audience segment.
    #[maybe_async::maybe_async]
    pub async fn get(&self, segment_id: &str) -> crate::Result<AudienceSegment> {
        let path = format!(
            "/audience/segments/{}",
            Config::encode_path_segment(segment_id)
        );
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ShowAudienceSegmentResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Update an audience segment.
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        segment_id: &str,
        options: UpdateAudienceSegmentOptions,
    ) -> crate::Result<AudienceSegment> {
        let path = format!(
            "/audience/segments/{}",
            Config::encode_path_segment(segment_id)
        );
        let request = self.0.build(Method::PATCH, &path).json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ShowAudienceSegmentResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Delete an audience segment.
    #[maybe_async::maybe_async]
    pub async fn delete(&self, segment_id: &str) -> crate::Result<()> {
        let path = format!(
            "/audience/segments/{}",
            Config::encode_path_segment(segment_id)
        );
        let request = self.0.build(Method::DELETE, &path);
        self.0.send(request).await?;
        Ok(())
    }
}

// ── Shared Condition Types ─────────────────────────────────────────────────

/// A single condition row within a segment condition group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentCondition {
    /// Field name being evaluated (e.g. `email`, `status`, or a property name).
    pub field: String,
    /// Comparison operator.
    pub operator: SegmentOperator,
    /// Comparison value. Omitted for boolean operators like `is_true`/`is_false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl SegmentCondition {
    /// Builds a condition with a value (`field op value`).
    pub fn new(
        field: impl Into<String>,
        operator: SegmentOperator,
        value: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            operator,
            value: Some(value.into()),
        }
    }

    /// Builds a value-less condition (for operators like `is_true`/`is_false`).
    pub fn unary(field: impl Into<String>, operator: SegmentOperator) -> Self {
        Self {
            field: field.into(),
            operator,
            value: None,
        }
    }
}

/// A group of conditions ANDed together within a segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentConditionGroup {
    /// Conditions in this group (joined with AND on the server).
    pub conditions: Vec<SegmentCondition>,
}

impl SegmentConditionGroup {
    /// Builds a new condition group from a list of conditions.
    pub fn new(conditions: Vec<SegmentCondition>) -> Self {
        Self { conditions }
    }
}

/// Top-level conditions structure sent on create/update.
///
/// Groups are ORed together; conditions within a group are ANDed.
#[derive(Debug, Clone, Serialize)]
pub struct SegmentConditionsInput {
    /// Condition groups (ORed).
    pub groups: Vec<SegmentConditionGroup>,
}

impl SegmentConditionsInput {
    /// Builds a new conditions input from a list of groups.
    pub fn new(groups: Vec<SegmentConditionGroup>) -> Self {
        Self { groups }
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for listing audience segments.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListAudienceSegmentsOptions {
    page: Option<u32>,
    per_page: Option<u32>,
    list_id: Option<String>,
}

impl ListAudienceSegmentsOptions {
    /// Creates new [`ListAudienceSegmentsOptions`] with default values.
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

    /// Filters segments by the list they're scoped to.
    #[inline]
    pub fn list_id(mut self, list_id: impl Into<String>) -> Self {
        self.list_id = Some(list_id.into());
        self
    }
}

/// Options for creating an audience segment.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct CreateAudienceSegmentOptions {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    list_id: Option<String>,
    conditions: SegmentConditionsInput,
}

impl CreateAudienceSegmentOptions {
    /// Creates new options with required name and conditions.
    pub fn new(name: impl Into<String>, conditions: SegmentConditionsInput) -> Self {
        Self {
            name: name.into(),
            list_id: None,
            conditions,
        }
    }

    /// Scopes the segment to a specific list.
    #[inline]
    pub fn with_list_id(mut self, list_id: impl Into<String>) -> Self {
        self.list_id = Some(list_id.into());
        self
    }
}

/// Options for updating an audience segment.
#[must_use]
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateAudienceSegmentOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    list_id: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conditions: Option<SegmentConditionsInput>,
}

impl UpdateAudienceSegmentOptions {
    /// Creates new options with no fields set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the new name.
    #[inline]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Scopes the segment to a list (use [`Self::clear_list_id`] to remove scoping).
    #[inline]
    pub fn with_list_id(mut self, list_id: impl Into<String>) -> Self {
        self.list_id = Some(Some(list_id.into()));
        self
    }

    /// Clears the list scoping by sending `null`.
    #[inline]
    pub fn clear_list_id(mut self) -> Self {
        self.list_id = Some(None);
        self
    }

    /// Sets the new conditions.
    #[inline]
    pub fn with_conditions(mut self, conditions: SegmentConditionsInput) -> Self {
        self.conditions = Some(conditions);
        self
    }
}

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListAudienceSegmentsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListAudienceSegmentsResponse,
}

/// Response from listing audience segments.
#[derive(Debug, Clone, Deserialize)]
pub struct ListAudienceSegmentsResponse {
    /// The audience segments.
    pub segments: Vec<AudienceSegment>,
    /// Pagination metadata.
    pub pagination: AudiencePagination,
}

#[derive(Debug, Deserialize)]
struct ShowAudienceSegmentResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: AudienceSegment,
}

/// An audience segment.
#[derive(Debug, Clone, Deserialize)]
pub struct AudienceSegment {
    /// Unique segment ID.
    pub id: String,
    /// Segment name.
    pub name: String,
    /// List this segment is scoped to, or `None` if it spans the whole audience.
    pub list_id: Option<String>,
    /// Name of the scoping list, if any.
    pub list_name: Option<String>,
    /// Condition groups defining segment membership.
    pub condition_groups: Vec<SegmentConditionGroup>,
    /// Cached count of contacts in this segment, if computed.
    pub cached_contacts_count: Option<u64>,
    /// Creation timestamp.
    pub created_at: String,
}
