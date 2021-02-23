use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TestLog {
    pub url: String,
    pub line_num: u32,
    pub url_raw: String,
    pub log_id: String,
    pub url_raw_display: String,
    pub url_html_display: String,
}

#[derive(Debug, Deserialize)]
pub struct EvgTest {
    pub task_id: String,
    pub status: String,
    pub test_file: String,
    pub exit_code: u16,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub logs: TestLog,
    pub duration: f64,
}
