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
pub mod templates;
pub mod webhooks;

pub mod services {
    //! Re-exports of all service types for convenient access.

    pub use super::domains::DomainsSvc;
    pub use super::emails::EmailsSvc;
    pub use super::templates::TemplatesSvc;
    pub use super::webhooks::WebhooksSvc;
}

pub mod types {
    //! Re-exports of commonly used request and response types.

    // Client
    pub use super::client::{AuthCheckData, AuthCheckResponse, HealthData, HealthResponse};

    // Emails
    pub use super::emails::{
        Attachment, CreateEmailOptions, EmailEvent, EmailEventDetail, EmailOptions,
        GetEmailResponse, ListEmailsOptions, ListEmailsResponse, Pagination, SendEmailResponse,
    };

    // Domains
    pub use super::domains::{
        CreateDomainResponse, DkimDnsRecord, DkimInfo, DnsRecords, Domain, DomainDetail,
    };

    // Webhooks
    pub use super::webhooks::Webhook;

    // Templates
    pub use super::templates::{
        CreateTemplateOptions, CreateTemplateResponse, ListTemplatesOptions, ListTemplatesResponse,
        MergeTag, Template, TemplatePagination,
    };

    // Errors
    pub use super::error::{ApiError, ValidationError};
}

/// Specialized [`Result`] type for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
