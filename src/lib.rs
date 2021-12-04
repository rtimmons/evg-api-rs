pub mod models;

use async_stream::stream;
use futures::stream::Stream;
use futures::stream::StreamExt;
use models::stats::EvgTaskStats;
use models::stats::EvgTaskStatsRequest;
use models::stats::EvgTestStats;
use models::stats::EvgTestStatsRequest;
use models::version::EvgVersion;
use models::{build::EvgBuild, patch::EvgPatch};
use models::{task::EvgTask, test::EvgTest};
use reqwest::{
    header::{HeaderMap, HeaderValue, LINK},
    Client, Response,
};
use serde::Deserialize;
use std::path::Path;
use std::{error::Error, fs};

const DEFAULT_CONFIG_FILE: &str = ".evergreen.yml";

#[derive(Debug, Deserialize, Clone)]
pub struct EvergreenConfigFile {
    pub user: String,
    pub api_key: String,
    pub api_server_host: String,
    ui_server_host: String,
}

pub fn get_evg_config() -> EvergreenConfigFile {
    let home = std::env::var("HOME").unwrap();
    let path = format!("{}/{}", home, DEFAULT_CONFIG_FILE);
    let contents = fs::read_to_string(Path::new(&path)).expect("Could not find config");
    let evg_config: EvergreenConfigFile =
        serde_yaml::from_str(&contents).expect("Could not read config");
    evg_config
}

#[derive(Clone)]
pub struct EvgClient {
    pub evg_config: EvergreenConfigFile,
    pub client: Client,
}

impl EvgClient {
    pub fn new() -> Result<EvgClient, Box<dyn Error>> {
        let evg_config = get_evg_config();

        let mut headers = HeaderMap::new();
        headers.insert("Api-User", HeaderValue::from_str(&evg_config.user).unwrap());
        headers.insert(
            "Api-Key",
            HeaderValue::from_str(&evg_config.api_key).unwrap(),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(EvgClient { evg_config, client })
    }

    fn build_url(&self, endpoint: &str, arg: &str) -> String {
        format!(
            "{}/rest/v2/{}/{}",
            self.evg_config.api_server_host, endpoint, arg
        )
    }

    pub async fn get_task(&self, task_id: &str) -> Result<EvgTask, Box<dyn Error>> {
        let url = self.build_url("tasks", task_id);
        let response = self.client.get(&url).send().await?;
        Ok(response.json().await?)
    }

    pub async fn get_version(&self, version_id: &str) -> Result<EvgVersion, Box<dyn Error>> {
        let url = self.build_url("versions", version_id);
        let response = self.client.get(&url).send().await?;
        Ok(response.json().await?)
    }

    pub async fn get_build(&self, build_id: &str) -> Result<Option<EvgBuild>, Box<dyn Error>> {
        let url = self.build_url("builds", build_id);
        let response = self.client.get(&url).send().await?;
        if response.status() == 404 {
            Ok(None)
        } else {
            Ok(Some(response.json().await?))
        }
    }

    pub async fn get_tests(&self, task_id: &str) -> Result<Vec<EvgTest>, Box<dyn Error>> {
        let url = format!("{}/tests", self.build_url("tasks", &task_id));
        let mut results: Vec<EvgTest> = vec![];
        let mut response = self.client.get(&url).send().await?;
        loop {
            let next_link = next_link(&response);
            let result_batch: Vec<EvgTest> = response.json().await?;
            results.extend(result_batch);

            if let Some(next) = next_link {
                response = self.client.get(&next).send().await?;
            } else {
                break;
            }
        }
        Ok(results)
    }

    pub async fn get_test_stats(&self, project_id: &str, query: &EvgTestStatsRequest) -> Result<Vec<EvgTestStats>, Box<dyn Error>> {
        let url = format!("{}/test_stats", self.build_url("projects", project_id));
        let response = self.client.get(&url).query(query).send().await?;

        Ok(response.json().await?)
    }

    pub async fn get_task_stats(&self, project_id: &str, query: &EvgTaskStatsRequest) -> Result<Vec<EvgTaskStats>, Box<dyn Error>> {
        let url = format!("{}/task_stats", self.build_url("projects", project_id));
        let response = self.client.get(&url).query(query).send().await?;
        Ok(response.json().await?)
    }

    pub async fn stream_build_tasks(&self, build_id: &str, status: Option<&str>) -> impl Stream<Item = EvgTask> {
        let mut url = format!("{}/tasks", self.build_url("builds", build_id));
        if let Some(s) = status {
            url = format!("{}?status={}", url, s);
        }
        let client = self.client.clone();

        stream! {
            let mut response = client.get(&url).send().await.unwrap();
            loop {
                let next_link = next_link(&response);
                let result_batch: Vec<EvgTask> = response.json().await.unwrap();
                for patch in result_batch {
                    yield patch;
                }

                if let Some(next) = next_link {
                    response = client.get(&next).send().await.unwrap();
                } else {
                    break;
                }
            }
        }
    }

    pub async fn stream_user_patches(&self, user_id: &str, limit: Option<usize>) -> impl Stream<Item = EvgPatch> {
        let mut url = format!("{}/patches", self.build_url("users", user_id));
        if let Some(l) = limit {
            url = format!("{}?limit={}", url, l);
        }
        let client = self.client.clone();

        stream! {
            let mut response = client.get(&url).send().await.unwrap();
            loop {
                let next_link = next_link(&response);
                let result_batch: Vec<EvgPatch> = response.json().await.unwrap();
                for patch in result_batch {
                    yield patch;
                }

                if let Some(next) = next_link {
                    response = client.get(&next).send().await.unwrap();
                } else {
                    break;
                }
            }
        }
    }

    pub async fn stream_project_patches(&self, project_id: &str, limit: Option<usize>) -> impl Stream<Item = EvgPatch> {
        let mut url = format!("{}/patches", self.build_url("projects", project_id));
        if let Some(l) = limit {
            url = format!("{}?limit={}", url, l);
        }
        let client = self.client.clone();

        stream! {
            let mut response = client.get(&url).send().await.unwrap();
            loop {
                let next_link = next_link(&response);
                let result_batch: Vec<EvgPatch> = response.json().await.unwrap();
                for patch in result_batch {
                    yield patch;
                }

                if let Some(next) = next_link {
                    response = client.get(&next).send().await.unwrap();
                } else {
                    break;
                }
            }
        }
    }

    pub fn stream_versions(&self, project_id: &str) -> impl Stream<Item = EvgVersion> {
        let url = format!(
            "{}/versions?requester=gitter_request",
            self.build_url("projects", project_id)
        );
        let client = self.client.clone();

        stream! {
            let mut response = client.get(&url).send().await.unwrap();
            loop {
                let next_link = next_link(&response);
                let result_batch: Vec<EvgVersion> = response.json().await.unwrap();
                for version in result_batch {
                    yield version;
                }

                if let Some(next) = next_link {
                    response = client.get(&next).send().await.unwrap();
                } else {
                    break;
                }
            }
        }
    }

    pub fn stream_log(&self, task: &EvgTask, log_name: &str) -> impl Stream<Item = String> {
        let task_log = format!("{}&text=true", task.logs.get(log_name).unwrap());
        let stream_future = self.client.get(&task_log).send();
        stream! {
            let mut stream = stream_future.await.unwrap().bytes_stream();
            while let Some(item) = stream.next().await {
                match item {
                    Ok(bytes) => {
                        let lines = std::str::from_utf8(&bytes).unwrap().split('\n');
                        for l in lines {
                            yield l.to_string();
                        }
                    }
                    _ => break,
                }
            }
        }
    }

    pub fn stream_test_log(&self, test: &EvgTest) -> impl Stream<Item = String> {
        let stream_future = self.client.get(&test.logs.url_raw).send();
        stream! {
            let mut stream = stream_future.await.unwrap().bytes_stream();
            while let Some(item) = stream.next().await {
                match item {
                    Ok(bytes) => {
                        let lines = std::str::from_utf8(&bytes).unwrap().split('\n');
                        for l in lines {
                            yield l.to_string();
                        }
                    }
                    _ => break,
                }
            }
        }
    }
}

fn next_link(response: &Response) -> Option<String> {
    if let Some(header) = response.headers().get(LINK) {
        let links = parse_link_header::parse(&header.to_str().unwrap()).unwrap();
        let next_link = links.get(&Some("next".to_string()));

        if let Some(link) = next_link {
            return Some(link.uri.to_string());
        }
    }
    None
}
