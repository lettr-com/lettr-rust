#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

pub use client::Lettr;
pub use emails::{Attachment, CreateEmailOptions};
pub use error::Error;

pub mod audience;
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

    pub use super::audience::{
        AudienceContactsSvc, AudienceListsSvc, AudiencePropertiesSvc, AudienceSegmentsSvc,
        AudienceSvc, AudienceTopicsSvc,
    };
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

    // Audience (shared)
    pub use super::audience::AudiencePagination;

    // Audience: lists
    pub use super::audience::lists::{
        AudienceList, BulkDeleteAudienceListsOptions, BulkDeleteAudienceListsResponse,
        CreateAudienceListOptions, ListAudienceListsOptions, ListAudienceListsResponse,
        UpdateAudienceListOptions,
    };

    // Audience: contacts
    pub use super::audience::contacts::{
        AudienceContact, AudienceContactListLink, AudienceContactStatus, AudienceContactTopicLink,
        BulkAttachContactsToListsResponse, BulkContactListMembershipOptions,
        BulkCreateAudienceContactsOptions, BulkCreateAudienceContactsResponse,
        BulkDetachContactsFromListsResponse, CreateAudienceContactOptions, DoubleOptInConfig,
        ListAudienceContactsOptions, ListAudienceContactsResponse, UpdateAudienceContactOptions,
        UpdateAudienceContactStatus,
    };

    // Audience: topics
    pub use super::audience::topics::{
        AudienceTopic, AudienceTopicDefaultSubscription, AudienceTopicVisibility,
        CreateAudienceTopicOptions, ListAudienceTopicsOptions, ListAudienceTopicsResponse,
        UpdateAudienceTopicOptions,
    };

    // Audience: properties
    pub use super::audience::properties::{
        AudienceProperty, AudiencePropertyType, CreateAudiencePropertyOptions,
        ListAudiencePropertiesOptions, ListAudiencePropertiesResponse,
        UpdateAudiencePropertyOptions,
    };

    // Audience: segments
    pub use super::audience::segments::{
        AudienceSegment, CreateAudienceSegmentOptions, ListAudienceSegmentsOptions,
        ListAudienceSegmentsResponse, SegmentCondition, SegmentConditionGroup,
        SegmentConditionsInput, SegmentOperator, UpdateAudienceSegmentOptions,
    };

    // Errors
    pub use super::error::{ApiError, ErrorCode, ValidationError};
}

/// Specialized [`Result`] type for [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
