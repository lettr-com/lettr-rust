//! Template folder listing.

use std::sync::Arc;

use reqwest::Method;
use serde::Deserialize;

use crate::config::Config;
use crate::templates::TemplatePurpose;

/// Service for the `/folders` endpoint.
///
/// Read-only: creating, renaming and deleting folders stay in the app, because
/// deleting one moves or deletes the templates inside it.
#[derive(Clone, Debug)]
pub struct FoldersSvc(pub(crate) Arc<Config>);

impl FoldersSvc {
    /// List the folders templates are filed into.
    ///
    /// This is what
    /// [`CreateTemplateOptions::with_folder_id`](crate::templates::CreateTemplateOptions::with_folder_id)
    /// was missing: nothing else in the SDK returns a folder id, so a caller
    /// either omitted it and accepted whichever folder the API picked, or
    /// hardcoded an integer read out of an app URL.
    ///
    /// Without a `project_id` the team's default project is used, the same way
    /// [`TemplatesSvc::list`](crate::templates::TemplatesSvc::list) resolves it.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::folders::ListFoldersOptions;
    /// # use lettr::templates::{CreateTemplateOptions, TemplatePurpose};
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let folders = client
    ///     .folders
    ///     .list(ListFoldersOptions::new().purpose(TemplatePurpose::Campaign))
    ///     .await?;
    ///
    /// if let Some(folder) = folders.folders.first() {
    ///     client
    ///         .templates
    ///         .create(
    ///             CreateTemplateOptions::new("October Newsletter")
    ///                 .with_json("{}")
    ///                 .with_folder_id(folder.id)
    ///                 .with_purpose(TemplatePurpose::Campaign),
    ///         )
    ///         .await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list(&self, options: ListFoldersOptions) -> crate::Result<ListFoldersResponse> {
        let mut request = self.0.build(Method::GET, "/folders");

        if let Some(project_id) = options.project_id {
            request = request.query(&[("project_id", project_id.to_string())]);
        }
        if let Some(ref purpose) = options.purpose {
            request = request.query(&[("purpose", purpose.as_str())]);
        }
        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }
        if let Some(page) = options.page {
            request = request.query(&[("page", page.to_string())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListFoldersResponseWrapper>().await?;
        Ok(wrapper.data)
    }
}

/// Options for listing folders.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListFoldersOptions {
    project_id: Option<u64>,
    purpose: Option<TemplatePurpose>,
    per_page: Option<u32>,
    page: Option<u32>,
}

impl ListFoldersOptions {
    /// Creates new [`ListFoldersOptions`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by project ID. If not set, uses the team's default project.
    #[inline]
    pub fn project_id(mut self, project_id: u64) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Narrows the list to one module. Both are returned if not set.
    #[inline]
    pub fn purpose(mut self, purpose: TemplatePurpose) -> Self {
        self.purpose = Some(purpose);
        self
    }

    /// Sets the number of results per page (1-100).
    #[inline]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Sets the page number.
    #[inline]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }
}

#[derive(Debug, Deserialize)]
struct ListFoldersResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListFoldersResponse,
}

/// Paginated list of folders.
#[derive(Debug, Clone, Deserialize)]
pub struct ListFoldersResponse {
    /// The folders on this page.
    pub folders: Vec<Folder>,
    /// Pagination metadata.
    pub pagination: FolderPagination,
}

/// A folder templates are filed into.
///
/// [`id`](Self::id) is what
/// [`CreateTemplateOptions::with_folder_id`](crate::templates::CreateTemplateOptions::with_folder_id)
/// expects, so listing folders is how a caller picks where a template lands
/// instead of hardcoding an integer read out of an app URL.
#[derive(Debug, Clone, Deserialize)]
pub struct Folder {
    /// Folder ID.
    pub id: u64,
    /// Folder name.
    pub name: String,
    /// Project this folder belongs to.
    pub project_id: u64,
    /// The module this folder belongs to. A template can only be filed into a
    /// folder of its own module.
    #[serde(default = "crate::templates::default_purpose")]
    pub purpose: TemplatePurpose,
    /// How many templates are in this folder.
    #[serde(default)]
    pub templates_count: u32,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// Pagination metadata for folder list responses.
#[derive(Debug, Clone, Deserialize)]
pub struct FolderPagination {
    /// Total number of folders.
    pub total: u64,
    /// Results per page.
    pub per_page: u32,
    /// Current page number.
    pub current_page: u32,
    /// Last page number.
    pub last_page: u32,
}
