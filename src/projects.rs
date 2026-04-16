use std::sync::Arc;

use reqwest::Method;
use serde::Deserialize;

use crate::config::Config;

/// Service for the `/projects` endpoints.
#[derive(Clone, Debug)]
pub struct ProjectsSvc(pub(crate) Arc<Config>);

impl ProjectsSvc {
    /// List all projects for your team.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::projects::ListProjectsOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let options = ListProjectsOptions::new().per_page(10);
    /// let response = client.projects.list(options).await?;
    ///
    /// for project in &response.projects {
    ///     println!("{}: {}", project.id, project.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list(
        &self,
        options: ListProjectsOptions,
    ) -> crate::Result<ListProjectsResponse> {
        let mut request = self.0.build(Method::GET, "/projects");

        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }
        if let Some(page) = options.page {
            request = request.query(&[("page", page.to_string())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListProjectsResponseWrapper>().await?;
        Ok(wrapper.data)
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for listing projects.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListProjectsOptions {
    per_page: Option<u32>,
    page: Option<u32>,
}

impl ListProjectsOptions {
    /// Creates new [`ListProjectsOptions`] with default values.
    pub fn new() -> Self {
        Self::default()
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

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListProjectsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListProjectsResponse,
}

/// Response from listing projects.
#[derive(Debug, Clone, Deserialize)]
pub struct ListProjectsResponse {
    /// List of projects.
    pub projects: Vec<Project>,
    /// Pagination information.
    pub pagination: ProjectsPagination,
}

/// A project.
#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    /// Project ID.
    pub id: u64,
    /// Project name.
    pub name: String,
    /// Project emoji.
    pub emoji: Option<String>,
    /// Team ID.
    pub team_id: u64,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// Pagination metadata for project list responses.
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectsPagination {
    /// Total number of projects.
    pub total: u64,
    /// Results per page.
    pub per_page: u32,
    /// Current page number.
    pub current_page: u32,
    /// Last page number.
    pub last_page: u32,
}
