use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BuildVariantStatus {
    pub build_variant: String,
    pub build_id: String,
}

#[derive(Debug, Deserialize)]
pub struct EvgVersion {
    pub version_id: String,
    pub create_time: DateTime<Utc>,
    pub start_time: Option<DateTime<Utc>>,
    pub finish_time: Option<DateTime<Utc>>,
    pub revision: String,
    pub order: u64,
    pub project: String,
    pub author: String,
    pub author_email: String,
    pub message: String,
    pub status: String,
    pub repo: String,
    pub branch: String,
    pub errors: Option<Vec<String>>,
    pub ignored: Option<bool>,
    pub requester: Option<String>,
    pub build_variants_status: Option<Vec<BuildVariantStatus>>,
}
