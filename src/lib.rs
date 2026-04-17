#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub use client::Lettr;
pub use emails::{Attachment, CreateEmailOptions};
pub use error::Error;

mod client;
pub(crate) mod config;
pub mod domains;
pub mod emails;
pub mod error;
pub mod projects;
pub mod templates;
pub mod webhooks;

pub mod services {
    //! Re-exports of all service types for convenient access.

    pub use super::domains::DomainsSvc;
    pub use super::emails::EmailsSvc;
    pub use super::projects::ProjectsSvc;
    pub use super::templates::TemplatesSvc;
    pub use super::webhooks::WebhooksSvc;
}

pub mod types {
    //! Re-exports of commonly used request and response types.

    // Client
    pub use super::client::{AuthCheckData, AuthCheckResponse, HealthData, HealthResponse};

    // Emails
    pub use super::emails::{
        Attachment, CreateEmailOptions, EmailEvent, EmailEventsData, EmailOptions, EmailState,
        EventType, GeoIp, GetEmailResponse, ListEmailEventsOptions, ListEmailEventsResponse,
        ListEmailsOptions, ListEmailsResponse, Pagination, QuotaInfo, ScheduleEmailOptions,
        ScheduledEmailState, ScheduledTransmission, SendEmailResponse, SendEmailWithQuotaResponse,
        SentEmailEventsData, SentEmailListItem, UserAgentParsed,
    };

    // Domains
    pub use super::domains::{
        CreateDomainResponse, DkimDnsRecord, DkimInfo, DmarcPolicy, DmarcValidationResult,
        DnsProvider, DnsRecords, DnsVerificationStatus, Domain, DomainDetail,
        DomainDnsVerification, DomainStatus, SpfValidationResult, VerifyDomainResponse,
    };

    // Webhooks
    pub use super::webhooks::{
        CreateWebhookOptions, UpdateWebhookOptions, Webhook, WebhookAuthType, WebhookEventsMode,
        WebhookStatus,
    };

    // Templates
    pub use super::templates::{
        CreateTemplateOptions, CreateTemplateResponse, GetTemplateHtmlResponse,
        ListTemplatesOptions, ListTemplatesResponse, MergeTag, MergeTagChild, MergeTagsList,
        Template, TemplateDetail, TemplatePagination, UpdateTemplateOptions,
        UpdateTemplateResponse,
    };

    // Projects
    pub use super::projects::{
        ListProjectsOptions, ListProjectsResponse, Project, ProjectsPagination,
    };

    // Errors
    pub use super::error::{ApiError, ErrorCode, ValidationError};
}

/// Specialized [`Result`] type for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
