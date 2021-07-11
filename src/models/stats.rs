use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct EvgTestStatsRequest {
    pub after_date: String,
    pub before_date: String,
    pub group_num_days: u64,
    pub variants: String,
    pub tasks: String,
    pub tests: Option<String>,
}

impl Default for EvgTestStatsRequest {
    fn default() -> Self {
        Self {
            after_date: String::from(""),
            before_date: String::from(""),
            group_num_days: 1,
            variants: String::from(""),
            tasks: String::from(""),
            tests: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EvgTestStats {
    pub test_file: String,
    pub task_name: String,
    pub variant: String,
    pub distro: Option<String>,
    pub date: String,
    pub num_pass: u64,
    pub num_fail: u64,
    pub avg_duration_pass: f64,
}

#[derive(Debug, Serialize)]
pub struct EvgTaskStatsRequest {
    pub after_date: String,
    pub before_date: String,
    pub group_num_days: u64,
    pub variants: String,
    pub tasks: String,
}

impl Default for EvgTaskStatsRequest {
    fn default() -> Self {
        Self {
            after_date: String::from(""),
            before_date: String::from(""),
            group_num_days: 1,
            variants: String::from(""),
            tasks: String::from(""),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct EvgTaskStats {
    pub task_name: String,
    pub variant: String,
    pub distro: Option<String>,
    pub date: String,
    pub num_success: u64,
    pub num_failed: u64,
    pub num_total: u64,
    pub num_timeout: u64,
    pub num_test_failed: u64,
    pub num_system_failed: u64,
    pub num_setup_failed: u64,
    pub avg_duration_success: f64,
}

impl EvgTaskStats {
    pub fn pass_rate(&self) -> f64 {
        self.num_success as f64 / self.num_total as f64
    }
}
