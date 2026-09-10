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
        if let Some(folder_id) = options.folder_id {
            request = request.query(&[("folder_id", folder_id.to_string())]);
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

    /// Retrieve details of a single template.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let template = client.templates.get("welcome-email", None).await?;
    /// println!("Name: {}, Active version: {:?}", template.name, template.active_version);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn get(&self, slug: &str, project_id: Option<u64>) -> crate::Result<TemplateDetail> {
        let path = format!("/templates/{slug}");
        let mut request = self.0.build(Method::GET, &path);

        if let Some(project_id) = project_id {
            request = request.query(&[("project_id", project_id.to_string())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<ShowTemplateResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Update an existing template.
    ///
    /// If `html` or `json` is provided, a new active version will be created.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # use lettr::templates::UpdateTemplateOptions;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let options = UpdateTemplateOptions::new()
    ///     .with_name("Updated Welcome Email")
    ///     .with_html("<h1>Hello {{NAME}}!</h1>");
    ///
    /// let result = client.templates.update("welcome-email", options).await?;
    /// println!("Updated: {}, Version: {}", result.name, result.active_version);
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn update(
        &self,
        slug: &str,
        options: UpdateTemplateOptions,
    ) -> crate::Result<UpdateTemplateResponse> {
        let path = format!("/templates/{slug}");
        let request = self.0.build(Method::PUT, &path).json(&options);
        let response = self.0.send(request).await?;
        let wrapper = response.json::<UpdateTemplateResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Delete a template.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// client.templates.delete("welcome-email", None).await?;
    /// println!("Template deleted.");
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn delete(&self, slug: &str, project_id: Option<u64>) -> crate::Result<()> {
        let path = format!("/templates/{slug}");
        let mut request = self.0.build(Method::DELETE, &path);

        if let Some(project_id) = project_id {
            request = request.query(&[("project_id", project_id.to_string())]);
        }

        self.0.send(request).await?;
        Ok(())
    }

    /// Get merge tags for a template version.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let tags = client.templates.get_merge_tags("welcome-email", None, None).await?;
    /// for tag in &tags.merge_tags {
    ///     println!("{}: required={}", tag.key, tag.required);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn get_merge_tags(
        &self,
        slug: &str,
        project_id: Option<u64>,
        version: Option<u32>,
    ) -> crate::Result<MergeTagsList> {
        let path = format!("/templates/{slug}/merge-tags");
        let mut request = self.0.build(Method::GET, &path);

        if let Some(project_id) = project_id {
            request = request.query(&[("project_id", project_id.to_string())]);
        }
        if let Some(version) = version {
            request = request.query(&[("version", version.to_string())]);
        }

        let response = self.0.send(request).await?;
        let wrapper = response.json::<GetMergeTagsResponseWrapper>().await?;
        Ok(wrapper.data)
    }

    /// Get rendered HTML for a template.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use lettr::Lettr;
    /// # async fn run() -> lettr::Result<()> {
    /// let client = Lettr::new("your-api-key");
    ///
    /// let result = client.templates.get_html(1, "welcome-email").await?;
    /// println!("HTML length: {}", result.html.len());
    /// # Ok(())
    /// # }
    /// ```
    #[maybe_async::maybe_async]
    pub async fn get_html(
        &self,
        project_id: u64,
        slug: &str,
    ) -> crate::Result<GetTemplateHtmlResponse> {
        let mut request = self.0.build(Method::GET, "/templates/html");
        request = request.query(&[
            ("project_id", project_id.to_string()),
            ("slug", slug.to_string()),
        ]);

        let response = self.0.send(request).await?;
        let wrapper = response.json::<GetTemplateHtmlResponseWrapper>().await?;
        Ok(wrapper.data)
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

/// Options for listing templates.
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct ListTemplatesOptions {
    project_id: Option<u64>,
    folder_id: Option<u64>,
    purpose: Option<TemplatePurpose>,
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

    /// Narrows the list to one folder of the resolved project.
    ///
    /// Discover ids with [`FoldersSvc::list`](crate::folders::FoldersSvc::list).
    /// One `per_page(100)` call reconciles a whole bulk import instead of a
    /// detail call per template, each of which drags the full HTML payload
    /// against the same rate limit.
    ///
    /// A folder that is not in the resolved project is a 404, not an empty
    /// list, so a typo cannot be misread as "nothing is there yet".
    #[inline]
    pub fn folder_id(mut self, folder_id: u64) -> Self {
        self.folder_id = Some(folder_id);
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

    /// Folder ID within the project. Must belong to the same module as
    /// `purpose`. Discover ids with
    /// [`FoldersSvc::list`](crate::folders::FoldersSvc::list).
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_id: Option<u64>,

    /// The module to create the template in. Omit to let the API decide,
    /// which today means transactional.
    #[serde(skip_serializing_if = "Option::is_none")]
    purpose: Option<TemplatePurpose>,
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
            purpose: None,
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

    /// Sets the module to create the template in.
    #[inline]
    pub fn with_purpose(mut self, purpose: TemplatePurpose) -> Self {
        self.purpose = Some(purpose);
        self
    }
}

/// Options for updating an existing template.
#[must_use]
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateTemplateOptions {
    /// Project ID to find the template in.
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<u64>,

    /// New name for the template.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// New HTML content. Creates a new active version.
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<String>,

    /// New JSON content for Topol editor. Creates a new active version.
    #[serde(skip_serializing_if = "Option::is_none")]
    json: Option<String>,
}

impl UpdateTemplateOptions {
    /// Creates new [`UpdateTemplateOptions`] with no fields set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the project ID.
    #[inline]
    pub fn with_project_id(mut self, project_id: u64) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Sets the new template name.
    #[inline]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the new HTML content.
    #[inline]
    pub fn with_html(mut self, html: impl Into<String>) -> Self {
        self.html = Some(html.into());
        self
    }

    /// Sets the new Topol editor JSON content.
    #[inline]
    pub fn with_json(mut self, json: impl Into<String>) -> Self {
        self.json = Some(json.into());
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
    pub folder_id: u64,
    /// The module this template belongs to.
    #[serde(default = "default_purpose")]
    pub purpose: TemplatePurpose,
    /// How far this template has got through preparation.
    #[serde(default = "default_preparation_status")]
    pub preparation_status: TemplatePreparationStatus,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

/// The module a template belongs to.
///
/// The two do not mix: only [`Campaign`](Self::Campaign) templates can be
/// picked by the campaign builder, and only [`Transactional`](Self::Transactional)
/// ones can be sent as single emails.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TemplatePurpose {
    Transactional,
    Campaign,
    /// An unknown purpose not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl TemplatePurpose {
    /// The wire value, for query parameters.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Transactional => "transactional",
            Self::Campaign => "campaign",
            Self::Unknown(s) => s,
        }
    }
}

impl std::fmt::Display for TemplatePurpose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The purpose an API that predates the field implied: there was no other kind.
pub(crate) fn default_purpose() -> TemplatePurpose {
    TemplatePurpose::Transactional
}

/// How far a template has got through preparation.
///
/// Creating or updating a template through the API defers image migration and
/// HTML rendering to a background job. On a create with JSON there is no HTML
/// at all until it finishes; on an **update** the previous render stays in
/// place, so the template is still sendable but is serving the *old* content.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TemplatePreparationStatus {
    Pending,
    Ready,
    Failed,
    /// An unknown status not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl TemplatePreparationStatus {
    /// Whether the content you last sent is the content that will go out.
    ///
    /// Deliberately not named `is_ready`: this is not the same question as
    /// "can I send this". A template being prepared after an update keeps its
    /// previous render and stays sendable, so wiring this into a send guard
    /// would block legitimate sends.
    #[must_use]
    pub fn is_settled(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

impl std::fmt::Display for TemplatePreparationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Ready => write!(f, "ready"),
            Self::Failed => write!(f, "failed"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// What an API that predates the field implied: every template with HTML was
/// simply usable, so `Ready` is the honest default. `Pending` would look like a
/// stalled queue and hang anything waiting for readiness.
fn default_preparation_status() -> TemplatePreparationStatus {
    TemplatePreparationStatus::Ready
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
    pub folder_id: u64,
    /// The module this template belongs to.
    #[serde(default = "default_purpose")]
    pub purpose: TemplatePurpose,
    /// How far this template has got through preparation.
    #[serde(default = "default_preparation_status")]
    pub preparation_status: TemplatePreparationStatus,
    /// Active version number.
    pub active_version: u32,
    /// Extracted merge tags.
    #[serde(default)]
    pub merge_tags: Vec<MergeTag>,
    /// Creation timestamp.
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct ShowTemplateResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: TemplateDetail,
}

/// Detailed template information.
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateDetail {
    /// Template ID.
    pub id: u64,
    /// Template name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// Project ID.
    pub project_id: u64,
    /// Folder ID.
    pub folder_id: u64,
    /// The module this template belongs to.
    #[serde(default = "default_purpose")]
    pub purpose: TemplatePurpose,
    /// How far this template has got through preparation.
    #[serde(default = "default_preparation_status")]
    pub preparation_status: TemplatePreparationStatus,
    /// Active version number.
    pub active_version: Option<u32>,
    /// Total number of versions.
    pub versions_count: u32,
    /// HTML content of the active version.
    #[serde(default)]
    pub html: Option<String>,
    /// JSON definition of the active version (for visual editor templates).
    #[serde(default)]
    pub json: Option<String>,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
struct UpdateTemplateResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: UpdateTemplateResponse,
}

/// Response from updating a template.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTemplateResponse {
    /// Template ID.
    pub id: u64,
    /// Template name.
    pub name: String,
    /// URL-friendly slug.
    pub slug: String,
    /// Project ID.
    pub project_id: u64,
    /// Folder ID.
    pub folder_id: u64,
    /// The module this template belongs to.
    #[serde(default = "default_purpose")]
    pub purpose: TemplatePurpose,
    /// How far this template has got through preparation.
    #[serde(default = "default_preparation_status")]
    pub preparation_status: TemplatePreparationStatus,
    /// Active version number.
    pub active_version: u32,
    /// Extracted merge tags.
    #[serde(default)]
    pub merge_tags: Vec<MergeTag>,
    /// Creation timestamp.
    pub created_at: String,
    /// Last update timestamp.
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
struct GetMergeTagsResponseWrapper {
    #[allow(dead_code)]
    message: String,
    data: MergeTagsList,
}

/// Merge tags for a template version.
#[derive(Debug, Clone, Deserialize)]
pub struct MergeTagsList {
    /// Project ID.
    pub project_id: u64,
    /// Template slug.
    pub template_slug: String,
    /// Template version number.
    pub version: u32,
    /// List of merge tags.
    pub merge_tags: Vec<MergeTag>,
}

/// A merge tag extracted from a template.
#[derive(Debug, Clone, Deserialize)]
pub struct MergeTag {
    /// The merge tag key.
    pub key: String,
    /// Whether this merge tag is required.
    pub required: bool,
    /// The data type of the merge tag (only present for loop children).
    #[serde(rename = "type", default)]
    pub merge_tag_type: Option<String>,
    /// Child merge tags for loop blocks.
    #[serde(default)]
    pub children: Option<Vec<MergeTagChild>>,
}

/// A child merge tag within a loop block.
#[derive(Debug, Clone, Deserialize)]
pub struct MergeTagChild {
    /// The child merge tag key.
    pub key: String,
    /// The data type of the child merge tag.
    #[serde(rename = "type", default)]
    pub merge_tag_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GetTemplateHtmlResponseWrapper {
    data: GetTemplateHtmlResponse,
}

/// Response from getting template HTML.
#[derive(Debug, Clone, Deserialize)]
pub struct GetTemplateHtmlResponse {
    /// The template HTML content.
    pub html: String,
    /// Merge tags in the template.
    pub merge_tags: Vec<TemplateHtmlMergeTag>,
    /// The template subject line, if set.
    #[serde(default)]
    pub subject: Option<String>,
}

/// A merge tag from the template HTML endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateHtmlMergeTag {
    /// The merge tag key.
    pub key: String,
    /// The merge tag display name.
    pub name: String,
    /// Whether this merge tag is required.
    pub required: bool,
}
