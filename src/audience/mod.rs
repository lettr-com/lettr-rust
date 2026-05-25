//! Audience management: lists, contacts, topics, properties, and segments.
//!
//! Access through the [`Lettr::audience`](crate::Lettr) field. The
//! [`AudienceSvc`] composite groups one sub-service per resource:
//!
//! ```rust,no_run
//! # use lettr::Lettr;
//! # async fn run() -> lettr::Result<()> {
//! let client = Lettr::new("your-api-key");
//!
//! let lists = client.audience.lists.list(Default::default()).await?;
//! let contacts = client.audience.contacts.list(Default::default()).await?;
//! # let _ = (lists, contacts);
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;

use serde::Deserialize;

use crate::config::Config;

pub mod contacts;
pub mod lists;
pub mod properties;
pub mod segments;
pub mod topics;

pub use contacts::AudienceContactsSvc;
pub use lists::AudienceListsSvc;
pub use properties::AudiencePropertiesSvc;
pub use segments::AudienceSegmentsSvc;
pub use topics::AudienceTopicsSvc;

/// Composite service exposing all `/audience` sub-resources.
#[derive(Clone, Debug)]
pub struct AudienceSvc {
    /// Audience lists (groupings of contacts).
    pub lists: AudienceListsSvc,
    /// Audience contacts (individual subscribers).
    pub contacts: AudienceContactsSvc,
    /// Audience topics (subscription categories).
    pub topics: AudienceTopicsSvc,
    /// Audience properties (custom fields).
    pub properties: AudiencePropertiesSvc,
    /// Audience segments (rule-based contact filters).
    pub segments: AudienceSegmentsSvc,
}

impl AudienceSvc {
    pub(crate) fn new(config: Arc<Config>) -> Self {
        Self {
            lists: AudienceListsSvc(Arc::clone(&config)),
            contacts: AudienceContactsSvc(Arc::clone(&config)),
            topics: AudienceTopicsSvc(Arc::clone(&config)),
            properties: AudiencePropertiesSvc(Arc::clone(&config)),
            segments: AudienceSegmentsSvc(Arc::clone(&config)),
        }
    }
}

/// Pagination metadata shared across all audience list responses.
#[derive(Debug, Clone, Deserialize)]
pub struct AudiencePagination {
    /// Total number of items.
    pub total: u64,
    /// Results per page.
    pub per_page: u32,
    /// Current page number.
    pub current_page: u32,
    /// Last page number.
    pub last_page: u32,
}
