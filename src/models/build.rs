use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct BuildStatusCounts {
    pub succeeded: u32,
    pub failed: u32,
    pub started: u32,
    pub undispatched: u32,
    pub inactivate: Option<u32>,
    pub dispatched: u32,
    pub timed_out: u32,
}

impl BuildStatusCounts {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add(&mut self, other: &Self) {
        self.succeeded += other.succeeded;
        self.failed += other.failed;
        self.started += other.started;
        self.undispatched += other.undispatched;
        self.dispatched += other.dispatched;
        self.timed_out += other.timed_out;
    }

    pub fn total_task_count(&self) -> u32 {
        self.undispatched
            + self.dispatched
            + self.started
            + self.failed
            + self.succeeded
            + self.timed_out
    }

    pub fn finished_task_count(&self) -> u32 {
        self.succeeded + self.failed + self.timed_out
    }

    pub fn pending_task_count(&self) -> u32 {
        self.started + self.undispatched
    }

    pub fn completed_task_count(&self) -> u32 {
        self.failed + self.succeeded + self.timed_out
    }

    pub fn percent_complete(&self) -> f64 {
        self.finished_task_count() as f64 / self.total_task_count() as f64
    }
}

#[derive(Debug, Deserialize, Clone)]
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
    pub status_counts: BuildStatusCounts,
}
