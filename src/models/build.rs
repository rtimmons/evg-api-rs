use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StatusCounts {
    pub succeeded: u32,
    pub failed: u32,
    pub started: u32,
    pub undispatched: u32,
    pub inactivate: Option<u32>,
    pub dispatched: u32,
    pub timed_out: u32,
}

#[derive(Debug, Deserialize)]
pub struct EvgBuild {
    #[serde(alias = "_id")]
    pub id: String, 
    pub project_id: String,
    pub create_time: Option<DateTime<Utc>>,
    pub start_time: Option<DateTime<Utc>>,
    pub finish_time: Option<DateTime<Utc>>,
    pub version: String,
    pub branch: Option<String>,
    pub git_hash: String,
    pub build_variant: String,
    pub status: String,
    pub activated: bool,
    pub activated_by: String,
    pub activated_time: Option<DateTime<Utc>>,
    pub order: u64,
    pub tasks: Vec<String>,
    pub time_taken_ms: u64,
    pub display_name: String,
    pub predicted_makespan_ms: u64,
    pub actual_makespan_ms: u64,
    pub origin: String,
    pub status_counts: StatusCounts
}
