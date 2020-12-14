use std::fs::File;
use std::path::{PathBuf, Path};
use std::io::{BufReader};

use futures::{stream, Stream, StreamExt};
use serde::{Serialize, Deserialize};
use reqwest::{Client, ClientBuilder, header, Response};

#[derive(Debug, PartialEq, Deserialize)]
struct CommitQueue {
    enabled: bool,
    merge_method: String,
    patch_type: String,
    message: String,
}

#[derive(Debug, PartialEq, Deserialize)]
struct TaskSync {
    config_enabled: bool,
    patch_enabled: bool,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Project {
    owner_name: String,
    repo_name: String,
    branch_name: String,
    repo_kind: String,
    enabled: bool,
    private: bool,
    batch_time: usize,
    remote_path: String,
    spawn_host_script_path: String,
    identifier: String,
    display_name: String,
    deactivate_previous: bool,
    tracks_push_events: bool,
    pr_testing_enabled: bool,
    git_tag_versions_enabled: bool,
    default_logger: String,
    tracked: bool,
    patching_disabled: bool,
    repotracker_disabled: bool,
    dispatching_disabled: bool,
    disabled_stats_cache: bool,
    admins: Vec<String>,
    commit_queue: CommitQueue,
    task_sync: TaskSync,

    //     "git_tag_authorized_users": [],
    //     "git_tag_authorized_teams": [],
    //     "notify_on_failure": true,
    //     "tags": [],
    //     "revision": null,
    //     "triggers": [],
    //     "aliases": null,
    //     "variables": {
    //       "vars": null,
    //       "private_vars": null,
    //       "restricted_vars": null
    //     },
    //     "workstation_config": {
    //       "setup_commands": null,
    //       "git_clone": false
    //     },
    //     "subscriptions": null
    //   },

}

#[derive(Serialize, Deserialize)]
struct EvergreenConfig {
    user: String,
    api_key: String,
    api_server_host: String,
    ui_server_host: String,
}

struct EvergreenResultsIter<T> {
    evg_client: Client,
    current_items: Option<Vec<T>>,
    current_index: usize,
    next_link: Option<String>,
}

fn get_link_header(response: &Response) -> Option<String> {
    let header_link = response.headers().get(header::LINK);
    match header_link {
        Some(link_value) => Some(link_value.to_str().unwrap().to_string()),
        None => None
    }
}

// impl Iterator for EvergreenResultsIter<T> {
//     type Item = T;
//
//     async fn next(&mut self) -> Option<Self::Item> {
//         if self.current_index == self.current_items.len() {
//             let next_link = &self.next_link;
//             self.current_index = match next_link {
//                 Some(next_link) => {
//                     self.current_index = 0;
//                     let response = self.evg_client.get(&next_link).send().await.unwrap();
//                     self.next_link = get_link_header(response);
//                     response.json().await.unwrap()
//                 },
//                 None => None,
//             }
//         }
//
//         let return_value = self.current_items[self.current_index];
//         self.current_index += 1;
//         return_value
//     }
// }

struct EvergreenApiClient {
    evg_config: EvergreenConfig,
    client: Client,
}


fn create_client(evg_config: &EvergreenConfig) -> Client {
    let mut headers = header::HeaderMap::new();
    headers.append("Api-User", header::HeaderValue::from_str(evg_config.user.as_str()).unwrap());
    headers.append("Api-Key", header::HeaderValue::from_str(evg_config.api_key.as_str()).unwrap());

    ClientBuilder::new()
        .default_headers(headers)
        .build().unwrap()
}

impl EvergreenApiClient {
    fn from_file(evg_config_path: &Path) -> EvergreenApiClient {
        let input = File::open(evg_config_path).unwrap();
        let evg_config: EvergreenConfig = serde_yaml::from_reader(BufReader::new(input)).unwrap();

        EvergreenApiClient {
            client: create_client(&evg_config),
            evg_config,
        }
    }

    async fn get_project_list(self) -> Vec<Project> {
        let client = create_client(&self.evg_config);
        let url = format!("{}/rest/v2/projects", self.evg_config.api_server_host);
        let response = client.get(&url).send().await.unwrap();
        println!("{:?}", response.headers());
        response.json().await.unwrap()
    }

    // async fn get_project_iter(self) -> EvergreenResultsIter<Project> {
    //     let url = self.host + "/rest/v2/projects";
    //     let response = self.client.get(&url).send().await.unwrap();
    //     println!("{:?}", response.headers());
    //     EvergreenResultsIter {
    //         evg_client: self.client,
    //         current_items: response.json().await.unwrap(),
    //         current_index: 0,
    //         next_link: get_link_header(&response)
    //     }
    // }

    fn get_project_stream(&self) -> Box<dyn Stream<Item=Project>> {
        let evg_config = self.evg_config;
        let start_url = format!("{}/rest/v2/projects", evg_config.api_server_host);

        Box::new(stream::unfold(Some(start_url), move |state| async {
            let url = match state {
                Some(s) => s,
                None => return None,
            };

            let resp = self.client.get(&url).send().await.unwrap();
            let next_lint = get_link_header(&resp);
            let proj_list: Vec<Project> = resp.json().await.unwrap();
            Some((stream::iter(proj_list), next_lint))
        }).flatten())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let evg_config_path = PathBuf::from("/home/dbradf/.evergreen.yml");
    let evg_client = EvergreenApiClient::from_file(evg_config_path.as_path());
    // let input = File::open(evg_config_path.as_path()).unwrap();
    //
    // let evg_config: EvergreenConfig = serde_yaml::from_reader(BufReader::new(input)).unwrap();
    //
    // let url = evg_config.api_server_host + "/rest/v2/projects";
    // println!("{}", url);
    // ({"Api-User": auth.username, "Api-Key": auth.api_key
    // let response = reqwest::get(&url).await?;

    // println!("{}", response.text().await?);

    // let project_list  = evg_client.get_project_list().await;
    // for p in project_list.iter() {
    //     println!("{:?}", p);
    //     // println!("{}", p.display_name);
    //     // print!("\t");
    //     // for admin in p.admins.iter() {
    //     //     print!("{}, ", admin);
    //     // }
    //     // println!();
    // }
    let project_stream = evg_client.get_project_stream().await;
    (*project_stream).for_each(|p| {
        println!("{}", p.display_name);
    });
    // let count = evg_client.get_project_iter().count();
    // println!("Found {} projects", count);
    Ok(())
}
