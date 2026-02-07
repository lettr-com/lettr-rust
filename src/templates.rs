use std::sync::Arc;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Service for the `/templates` endpoints.
#[derive(Clone, Debug)]
pub struct TemplatesSvc(pub(crate) Arc<Config>);

impl TemplatesSvc {
    /// List email templates with optional pagination.
    ///
    /// If `project_id` is not provided, templates from the team's default project
    /// will be returned.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::templates::ListTemplatesOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let options = ListTemplatesOptions::new().per_page(10);
    /// let response = client.templates.list(options).await?;
    ///
    /// for template in &response.templates {
    ///     println!("{}: {} (slug: {})", template.id, template.name, template.slug);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn list(
        &self,
        options: ListTemplatesOptions,
    ) -> crate::Result<ListTemplatesResponse> {
        let mut request = self.0.build(Method::GET, "/templates");

        if let Some(project_id) = options.project_id {
            request = request.query(&[("project_id", project_id.to_string())]);
        }
        if let Some(per_page) = options.per_page {
            request = request.query(&[("per_page", per_page.to_string())]);
        }
        if let Some(page) = options.page {
            request = request.query(&[("page", page.to_string())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<ListTemplatesResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Create a new email template.
    ///
    /// Provide either HTML or Topol editor JSON content (but not both).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::templates::CreateTemplateOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let template = CreateTemplateOptions::new("Welcome Email")
    ///     .with_html("<h1>Hello {{FIRST_NAME}}!</h1>");
    ///
    /// let result = client.templates.create(template).await?;
    /// println!("Template created: {} (slug: {})", result.id, result.slug);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn create(
        &self,
        options: CreateTemplateOptions,
    ) -> crate::Result<CreateTemplateResponse> {
        let request = self.0.build(Method::POST, "/templates").json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<CreateTemplateResponseWrapper>().await?;
        Ok(wrapper.data)
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for listing templates.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListTemplatesOptions {
    project_id: Option<u64>,
    per_page: Option<u32>,
    page: Option<u32>,
}

impl ListTemplatesOptions {
    /// Creates new [`ListTemplatesOptions`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by project ID. If not set, uses the team's default project.
    #[inline]
    pub fn project_id(mut self, project_id: u64) -> Self {
        self.project_id = Some(project_id);
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

/// Options for creating a new template.
#[must_use]
#[derive(Debug, Clone, Serialize)]
pub struct CreateTemplateOptions {
    /// Template name.
    name: String,

    /// HTML content for the template.
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<String>,

    /// Topol editor JSON content.
    #[serde(skip_serializing_if = "Option::is_none")]
    json: Option<String>,

    /// Project ID. If not set, uses the team's default project.
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<u64>,

    /// Folder ID within the project.
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_id: Option<u64>,
}

impl CreateTemplateOptions {
    /// Creates new [`CreateTemplateOptions`] with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            html: None,
            json: None,
            project_id: None,
            folder_id: None,
        }
    }

    /// Sets the HTML content for the template.
    #[inline]
    pub fn with_html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    /// Sets the Topol editor JSON content for the template.
    #[inline]
    pub fn with_json(mut self, json: impl Into<String>) -> Self {
        self.json = Some(json.into());
        self
    }

    /// Sets the project ID.
    #[inline]
    pub fn with_project_id(mut self, project_id: u64) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Sets the folder ID.
    #[inline]
    pub fn with_folder_id(mut self, folder_id: u64) -> Self {
        self.folder_id = Some(folder_id);
        self
    }
}

// ── Response Types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListTemplatesResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: ListTemplatesResponse,
}

/// Response from listing templates.
#[derive(Debug, Clone, Deserialize)]
pub struct ListTemplatesResponse {
    /// List of templates.
    pub templates: Vec<Template>,
    /// Pagination information.
    pub pagination: TemplatePagination,
}

/// An email template.
#[derive(Debug, Clone, Deserialize)]
pub struct Template {
    /// Template ID.
    pub id: u64,
    /// Template name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// Project ID this template belongs to.
    pub project_id: u64,
    /// Folder ID this template belongs to.
    pub folder_id: Option<u64>,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// Pagination metadata for template list responses.
#[derive(Debug, Clone, Deserialize)]
pub struct TemplatePagination {
    /// Total number of templates.
    pub total: u64,
    /// Results per page.
    pub per_page: u32,
    /// Current page number.
    pub current_page: u32,
    /// Last page number.
    pub last_page: u32,
}

#[derive(Debug, Deserialize)]
struct CreateTemplateResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: CreateTemplateResponse,
}

/// Response from creating a template.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTemplateResponse {
    /// Template ID.
    pub id: u64,
    /// Template name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// Project ID.
    pub project_id: u64,
    /// Folder ID.
    pub folder_id: Option<u64>,
    /// Active version number.
    pub active_version: u32,
    /// Extracted merge tags.
    #[serde(default)]
    pub merge_tags: Vec<MergeTag>,
    /// Creation timestamp.
    pub created_at: String,
}

/// A merge tag extracted from a template.
#[derive(Debug, Clone, Deserialize)]
pub struct MergeTag {
    /// The merge tag key.
    pub key: String,
    /// Whether this merge tag is required.
    pub required: bool,
}
