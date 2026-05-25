use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::audience::AudiencePagination;
use crate::config::Config;

/// Service for the `/audience/lists` endpoints.
#[derive(Clone, Debug)]
pub struct AudienceListsSvc(pub(crate) Arc<Config>);

impl AudienceListsSvc {
    /// List audience lists.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::audience::lists::ListAudienceListsOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let options = ListAudienceListsOptions::new().per_page(50);
    /// let response = client.audience.lists.list(options).await?;
    ///
    /// for list in &response.lists {
    ///     println!("{}: {} ({} contacts)", list.id, list.name, list.contacts_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list(
        &self,
        options: ListAudienceListsOptions,
    ) -> crate::Result<ListAudienceListsResponse> {
        let mut request = self.0.build(Method::GET, "/audience/lists");

        if let Some(page) = options.page {
            request = request.query(&[("page", page.to_string())]);
        }
        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListAudienceListsResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Create a new audience list.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::audience::lists::CreateAudienceListOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let list = client.audience.lists.create(
    ///     CreateAudienceListOptions::new("Newsletter Subscribers")
    /// ).await?;
    /// println!("Created list: {}", list.id);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn create(&self, options: CreateAudienceListOptions) -> crate::Result<AudienceList> {
        let request = self.0.build(Method::POST, "/audience/lists").json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowAudienceListResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Retrieve a single audience list.
    #[maybe_async::maybe_async]
    pub async fn get(&self, list_id: &str) -> crate::Result<AudienceList> {
        let path = format!("/audience/lists/{}", Config::encode_path_segment(list_id));
        let request = self.0.build(Method::GET, &path);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowAudienceListResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Update an audience list.
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        list_id: &str,
        options: UpdateAudienceListOptions,
    ) -> crate::Result<AudienceList> {
        let path = format!("/audience/lists/{}", Config::encode_path_segment(list_id));
        let request = self.0.build(Method::PATCH, &path).json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowAudienceListResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Delete an audience list.
    #[maybe_async::maybe_async]
    pub async fn delete(&self, list_id: &str) -> crate::Result<()> {
        let path = format!("/audience/lists/{}", Config::encode_path_segment(list_id));
        let request = self.0.build(Method::DELETE, &path);
        self.0.send(request).await?;
        Ok(())
    }

    /// Delete multiple audience lists in a single request (1–50 IDs).
    #[maybe_async::maybe_async]
    pub async fn bulk_delete(
        &self,
        options: BulkDeleteAudienceListsOptions,
    ) -> crate::Result<BulkDeleteAudienceListsResponse> {
        let request = self
            .0
            .build(Method::DELETE, "/audience/lists/bulk")
            .json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response
            .json::<BulkDeleteAudienceListsResponseWrapper>()
            .await?;
        Ok(wrapper.data)
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for listing audience lists.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListAudienceListsOptions {
    page: Option<u32>,
    per_page: Option<u32>,
}

impl ListAudienceListsOptions {
    /// Creates new [`ListAudienceListsOptions`] with default values.
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

/// Options for creating an audience list.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct CreateAudienceListOptions {
    name: String,
}

impl CreateAudienceListOptions {
    /// Creates new [`CreateAudienceListOptions`] with the required name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Options for updating an audience list.
#[must_use]
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateAudienceListOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl UpdateAudienceListOptions {
    /// Creates new [`UpdateAudienceListOptions`] with no fields set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the new name for the list.
    #[inline]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Options for bulk-deleting audience lists.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct BulkDeleteAudienceListsOptions {
    list_ids: Vec<String>,
}

impl BulkDeleteAudienceListsOptions {
    /// Creates new options from a vector of list IDs (1–50 items).
    pub fn new(list_ids: Vec<String>) -> Self {
        Self { list_ids }
    }
}

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListAudienceListsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListAudienceListsResponse,
}

/// Response from listing audience lists.
#[derive(Debug, Clone, Deserialize)]
pub struct ListAudienceListsResponse {
    /// The audience lists.
    pub lists: Vec<AudienceList>,
    /// Pagination metadata.
    pub pagination: AudiencePagination,
}

#[derive(Debug, Deserialize)]
struct ShowAudienceListResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: AudienceList,
}

/// An audience list.
#[derive(Debug, Clone, Deserialize)]
pub struct AudienceList {
    /// Unique list ID.
    pub id: String,
    /// List name.
    pub name: String,
    /// Number of contacts in this list.
    pub contacts_count: u64,
}

#[derive(Debug, Deserialize)]
struct BulkDeleteAudienceListsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: BulkDeleteAudienceListsResponse,
}

/// Response from bulk-deleting audience lists.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkDeleteAudienceListsResponse {
    /// Number of lists actually deleted.
    pub deleted: u32,
}
