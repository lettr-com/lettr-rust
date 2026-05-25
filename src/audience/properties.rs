use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::audience::AudiencePagination;
use crate::config::Config;

// ── Enum Types ────────────────────────────────────────────────────────────

/// The data type of an audience property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AudiencePropertyType {
    String,
    Number,
    Boolean,
    Date,
    Json,
    /// An unknown property type not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for AudiencePropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Number => write!(f, "number"),
            Self::Boolean => write!(f, "boolean"),
            Self::Date => write!(f, "date"),
            Self::Json => write!(f, "json"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Service for the `/audience/properties` endpoints.
#[derive(Clone, Debug)]
pub struct AudiencePropertiesSvc(pub(crate) Arc<Config>);

impl AudiencePropertiesSvc {
    /// List audience properties.
    #[maybe_async::maybe_async]
    pub async fn list(
        &self,
        options: ListAudiencePropertiesOptions,
    ) -> crate::Result<ListAudiencePropertiesResponse> {
        let mut request = self.0.build(Method::GET, "/audience/properties");

        if let Some(page) = options.page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ListAudiencePropertiesResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Create a new audience property.
    ///
    /// `name` must match `^[a-z][a-z0-9_]*$`.
    #[maybe_async::maybe_async]
    pub async fn create(
        &self,
        options: CreateAudiencePropertyOptions,
    ) -> crate::Result<AudienceProperty> {
        let request = self
            .0
            .build(Method::POST, "/audience/properties")
            .json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ShowAudiencePropertyResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Retrieve a single audience property.
    #[maybe_async::maybe_async]
    pub async fn get(&self, property_id: &str) -> crate::Result<AudienceProperty> {
        let path = format!(
            "/audience/properties/{}",
            Config::encode_path_segment(property_id)
        );
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ShowAudiencePropertyResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Update an audience property. Only `fallback_value` is mutable.
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        property_id: &str,
        options: UpdateAudiencePropertyOptions,
    ) -> crate::Result<AudienceProperty> {
        let path = format!(
            "/audience/properties/{}",
            Config::encode_path_segment(property_id)
        );
        let request = self.0.build(Method::PATCH, &path).json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<ShowAudiencePropertyResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }

    /// Delete an audience property.
    #[maybe_async::maybe_async]
    pub async fn delete(&self, property_id: &str) -> crate::Result<()> {
        let path = format!(
            "/audience/properties/{}",
            Config::encode_path_segment(property_id)
        );
        let request = self.0.build(Method::DELETE, &path);
        self.0.send(request).await?;
        Ok(())
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for listing audience properties.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListAudiencePropertiesOptions {
    page: Option<u32>,
    per_page: Option<u32>,
}

impl ListAudiencePropertiesOptions {
    /// Creates new [`ListAudiencePropertiesOptions`] with default values.
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

/// Options for creating an audience property.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct CreateAudiencePropertyOptions {
    name: String,
    #[serde(rename = "type")]
    property_type: AudiencePropertyType,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback_value: Option<String>,
}

impl CreateAudiencePropertyOptions {
    /// Creates new [`CreateAudiencePropertyOptions`] with required name and type.
    pub fn new(name: impl Into<String>, property_type: AudiencePropertyType) -> Self {
        Self {
            name: name.into(),
            property_type,
            fallback_value: None,
        }
    }

    /// Sets the fallback value used when a contact has no value for this property.
    #[inline]
    pub fn with_fallback_value(mut self, fallback_value: impl Into<String>) -> Self {
        self.fallback_value = Some(fallback_value.into());
        self
    }
}

/// Options for updating an audience property.
///
/// Note: `name` and `type` are immutable per the API spec — only the fallback
/// value can be changed.
#[must_use]
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateAudiencePropertyOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback_value: Option<Option<String>>,
}

impl UpdateAudiencePropertyOptions {
    /// Creates new [`UpdateAudiencePropertyOptions`] with no fields set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the fallback value (use [`Self::clear_fallback_value`] to send null).
    #[inline]
    pub fn with_fallback_value(mut self, fallback_value: impl Into<String>) -> Self {
        self.fallback_value = Some(Some(fallback_value.into()));
        self
    }

    /// Clears the fallback value by sending `null`.
    #[inline]
    pub fn clear_fallback_value(mut self) -> Self {
        self.fallback_value = Some(None);
        self
    }
}

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListAudiencePropertiesResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListAudiencePropertiesResponse,
}

/// Response from listing audience properties.
#[derive(Debug, Clone, Deserialize)]
pub struct ListAudiencePropertiesResponse {
    /// The audience properties.
    pub properties: Vec<AudienceProperty>,
    /// Pagination metadata.
    pub pagination: AudiencePagination,
}

#[derive(Debug, Deserialize)]
struct ShowAudiencePropertyResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: AudienceProperty,
}

/// An audience property.
#[derive(Debug, Clone, Deserialize)]
pub struct AudienceProperty {
    /// Unique property ID.
    pub id: String,
    /// Property name (used as the key in `properties` maps on contacts).
    pub name: String,
    /// Data type.
    #[serde(rename = "type")]
    pub property_type: AudiencePropertyType,
    /// Fallback value when a contact has no value for this property.
    pub fallback_value: Option<String>,
    /// Creation timestamp.
    pub created_at: String,
}
