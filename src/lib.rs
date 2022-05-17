pub mod models;

use async_stream::stream;
use async_trait::async_trait;
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
use std::pin::Pin;
use std::{error::Error, fs};

pub type BoxedStream<T> = Pin<Box<dyn Stream<Item = T>>>;
pub type EvgError = Box<dyn Error + Sync + Send>;

const DEFAULT_CONFIG_FILE: &str = ".evergreen.yml";

#[derive(Debug, Deserialize, Clone)]
struct EvergreenConfigFile {
    pub user: String,
    pub api_key: String,
    pub api_server_host: String,
}

fn get_evg_config(path: &Path) -> Result<EvergreenConfigFile, EvgError> {
    let contents = fs::read_to_string(&path)?;
    let evg_config: EvergreenConfigFile = serde_yaml::from_str(&contents)?;
    Ok(evg_config)
}

#[async_trait]
pub trait EvgApiClient: Sync + Send {
    /// Get details about the given task.
    async fn get_task(&self, task_id: &str) -> Result<EvgTask, EvgError>;
    /// Get details about the given version.
    async fn get_version(&self, version_id: &str) -> Result<EvgVersion, EvgError>;
    /// Get details about the given build.
    async fn get_build(&self, build_id: &str) -> Result<Option<EvgBuild>, EvgError>;
    /// Get the tests belonging to the given task.
    async fn get_tests(&self, task_id: &str) -> Result<Vec<EvgTest>, EvgError>;
    /// Get test stats for the given query.
    async fn get_test_stats(
        &self,
        project_id: &str,
        query: &EvgTestStatsRequest,
    ) -> Result<Vec<EvgTestStats>, EvgError>;
    /// Get task stats for the given query.
    async fn get_task_stats(
        &self,
        project_id: &str,
        query: &EvgTaskStatsRequest,
    ) -> Result<Vec<EvgTaskStats>, EvgError>;
    /// Stream version of an evergreen project.
    fn stream_versions(&self, project_id: &str) -> BoxedStream<EvgVersion>;
    /// Stream user patches of an evergreen project.
    fn stream_user_patches(&self, user_id: &str, limit: Option<usize>) -> BoxedStream<EvgPatch>;
    /// Stream patches of an evergreen project.
    fn stream_project_patches(
        &self,
        project_id: &str,
        limit: Option<usize>,
    ) -> BoxedStream<EvgPatch>;
    /// Stream tasks of an evergreen build.
    fn stream_build_tasks(&self, build_id: &str, status: Option<&str>) -> BoxedStream<EvgTask>;
    /// Stream the contents of a task level log.
    fn stream_log(&self, task: &EvgTask, log_name: &str) -> BoxedStream<String>;
    /// Stream the contents of a test level log.
    fn stream_test_log(&self, test: &EvgTest) -> BoxedStream<String>;
}

#[derive(Clone)]
pub struct EvgClient {
    evg_config: EvergreenConfigFile,
    client: Client,
}

impl EvgClient {
    /// Create a new EvgClient based on the default evergreen auth file location (~/.evergreen.yml).
    pub fn new() -> Result<EvgClient, EvgError> {
        let home = std::env::var("HOME")?;
        let path = format!("{}/{}", home, DEFAULT_CONFIG_FILE);
        Self::from_file(Path::new(&path))
    }

    /// Create a new EvgClient based on the evergreen auth file at the provided location.
    pub fn from_file(config_file: &Path) -> Result<EvgClient, EvgError> {
        let evg_config = get_evg_config(config_file)?;
        let mut headers = HeaderMap::new();
        headers.insert("Api-User", HeaderValue::from_str(&evg_config.user)?);
        headers.insert("Api-Key", HeaderValue::from_str(&evg_config.api_key)?);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(EvgClient { evg_config, client })
    }

    fn build_url(&self, endpoint: &str, arg: &str) -> String {
        let out = format!(
            "{}/rest/v2/{}/{}",
            self.evg_config.api_server_host, endpoint, arg
        );
        print!("Ryan: {}", out);
        return out;
    }
}

#[async_trait]
impl EvgApiClient for EvgClient {
    async fn get_task(&self, task_id: &str) -> Result<EvgTask, EvgError> {
        let url = self.build_url("tasks", task_id);
        let response = self.client.get(&url).send().await?;
        Ok(response.json().await?)
    }

    async fn get_version(&self, version_id: &str) -> Result<EvgVersion, EvgError> {
        let url = self.build_url("versions", version_id);
        let response = self.client.get(&url).send().await?;
        Ok(response.json().await?)
    }

    async fn get_build(&self, build_id: &str) -> Result<Option<EvgBuild>, EvgError> {
        let url = self.build_url("builds", build_id);
        let response = self.client.get(&url).send().await?;
        if response.status() == 404 {
            Ok(None)
        } else {
            Ok(Some(response.json().await?))
        }
    }

    async fn get_tests(&self, task_id: &str) -> Result<Vec<EvgTest>, EvgError> {
        let url = format!("{}/tests", self.build_url("tasks", task_id));
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

    async fn get_test_stats(
        &self,
        project_id: &str,
        query: &EvgTestStatsRequest,
    ) -> Result<Vec<EvgTestStats>, EvgError> {
        let url = format!("{}/test_stats", self.build_url("projects", project_id));
        let response = self.client.get(&url).query(query).send().await?;

        Ok(response.json().await?)
    }

    async fn get_task_stats(
        &self,
        project_id: &str,
        query: &EvgTaskStatsRequest,
    ) -> Result<Vec<EvgTaskStats>, EvgError> {
        let url = format!("{}/task_stats", self.build_url("projects", project_id));
        let response = self.client.get(&url).query(query).send().await?;
        Ok(response.json().await?)
    }

    fn stream_versions(&self, project_id: &str) -> BoxedStream<EvgVersion> {
        let url = format!(
            "{}/versions?requester=gitter_request",
            self.build_url("projects", project_id)
        );
        let client = self.client.clone();

        Box::pin(stream! {
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
        })
    }

    fn stream_user_patches(&self, user_id: &str, limit: Option<usize>) -> BoxedStream<EvgPatch> {
        let mut url = format!("{}/patches", self.build_url("users", user_id));
        if let Some(l) = limit {
            url = format!("{}?limit={}", url, l);
        }
        let client = self.client.clone();

        Box::pin(stream! {
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
        })
    }

    fn stream_project_patches(
        &self,
        project_id: &str,
        limit: Option<usize>,
    ) -> BoxedStream<EvgPatch> {
        let mut url = format!("{}/patches", self.build_url("projects", project_id));
        if let Some(l) = limit {
            url = format!("{}?limit={}", url, l);
        }
        let client = self.client.clone();

        Box::pin(stream! {
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
        })
    }

    fn stream_build_tasks(&self, build_id: &str, status: Option<&str>) -> BoxedStream<EvgTask> {
        let mut url = format!("{}/tasks", self.build_url("builds", build_id));
        if let Some(s) = status {
            url = format!("{}?status={}", url, s);
        }
        let client = self.client.clone();

        Box::pin(stream! {
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
        })
    }

    fn stream_log(&self, task: &EvgTask, log_name: &str) -> BoxedStream<String> {
        let task_log = format!("{}&text=true", task.logs.get(log_name).unwrap());
        let stream_future = self.client.get(&task_log).send();
        Box::pin(stream! {
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
        })
    }

    fn stream_test_log(&self, test: &EvgTest) -> BoxedStream<String> {
        let stream_future = self.client.get(&test.logs.url_raw).send();

        Box::pin(stream! {
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
        })
    }
}

fn next_link(response: &Response) -> Option<String> {
    if let Some(header) = response.headers().get(LINK) {
        let links = parse_link_header::parse(header.to_str().unwrap()).unwrap();
        let next_link = links.get(&Some("next".to_string()));

        return next_link.map(|l| l.uri.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::response::Builder;
    use reqwest::Response;

    #[test]
    fn test_next_link_should_return_link_if_it_exists() {
        let response = Response::from(Builder::new()
            .status(200)
            .header(LINK, String::from("<https://evergreen.mongodb.com/rest/v2/tasks/task_id/tests?limit=100&start_at=abc123>; rel=\"next\""))
            .body("foo")
            .unwrap());

        let next_link = next_link(&response);

        assert_eq!(next_link, Some(String::from("https://evergreen.mongodb.com/rest/v2/tasks/task_id/tests?limit=100&start_at=abc123")));
    }

    #[test]
    fn test_next_link_should_return_none_if_no_next() {
        let response = Response::from(Builder::new()
            .status(200)
            .header(LINK, String::from("<https://evergreen.mongodb.com/rest/v2/tasks/task_id/tests?limit=100&start_at=abc123>; rel=\"other\""))
            .body("foo")
            .unwrap());

        let next_link = next_link(&response);

        assert_eq!(next_link, None);
    }

    #[test]
    fn test_next_link_should_return_none_if_no_link_header() {
        let response = Response::from(Builder::new()
            .status(200)
            .header("something", String::from("<https://evergreen.mongodb.com/rest/v2/tasks/task_id/tests?limit=100&start_at=abc123>; rel=\"other\""))
            .body("foo")
            .unwrap());

        let next_link = next_link(&response);

        assert_eq!(next_link, None);
    }
}
