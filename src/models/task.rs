use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct EvgTaskArtifact {
    pub name: String,
    pub url: String,
    pub visibility: String,
    pub ignore_for_fetch: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EvgTaskStatusDetails {
    pub status: String,
    #[serde(alias = "type")]
    pub status_type: String,
    pub desc: String,
    pub timed_out: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EvgTask {
    pub activated: bool,
    pub activated_by: String,
    pub artifacts: Option<Vec<EvgTaskArtifact>>,
    pub build_id: String,
    pub build_variant: String,
    pub create_time: DateTime<Utc>,
    // depends_on: Optional[List[Union[str, DisplayTaskDependency]]]
    pub dispatch_time: Option<DateTime<Utc>>,
    pub display_name: String,
    pub display_only: bool,
    pub distro_id: String,
    pub est_wait_to_start_ms: u32,
    pub execution: u32,
    pub execution_tasks: Option<Vec<String>>,
    pub expected_duration_ms: u64,
    pub finish_time: Option<DateTime<Utc>>,
    pub generate_task: bool,
    pub generated_by: String,
    pub host_id: String,
    pub ingest_time: Option<DateTime<Utc>>,
    pub logs: HashMap<String, String>,
    pub mainline: Option<bool>,
    pub order: u64,
    pub project_id: String,
    pub priority: u32,
    pub restarts: Option<u32>,
    pub revision: String,
    pub scheduled_time: Option<DateTime<Utc>>,
    pub start_time: Option<DateTime<Utc>>,
    pub status: String,
    pub status_details: EvgTaskStatusDetails,
    pub task_group: Option<String>,
    pub task_group_max_hosts: Option<u16>,
    pub task_id: String,
    pub time_taken_ms: u64,
    pub version_id: String,
}
