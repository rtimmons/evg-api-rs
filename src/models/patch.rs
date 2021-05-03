use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EvgPatch {
    pub patch_id: String,
    pub description: String,
    pub project_id: String,
    pub project_identifier: String,
    pub branch: String,
    pub git_hash: String,
    pub patch_number: u64,
    pub author: String,
    pub version: String,
    pub status: String,
    pub create_time: DateTime<Utc>,
    pub start_time: Option<DateTime<Utc>>,
    pub finish_time: Option<DateTime<Utc>>,
}
