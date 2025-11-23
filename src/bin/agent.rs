#![allow(clippy::suspicious_else_formatting)]

use ::time::Duration;
use axum_login::tracing::{debug, error, info, trace};
use clap::Parser;
use docker_api::Docker;
use docker_api::models::{ContainerState, ContainerStateStatusInlineItem};
use docker_api::opts::ContainerListOpts;
use futures::{Stream, StreamExt};
use nosqlensiie::nosql::model::VictoriaMetric;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{thread, time};
use tokio::task::JoinHandle;
#[cfg(unix)]
pub fn new_docker() -> Result<Docker, Box<dyn std::error::Error>> {
    Ok(Docker::unix("/var/run/docker.sock"))
}

#[cfg(not(unix))]
pub fn new_docker() -> Result<Docker, _> {
    Docker::new("tcp://127.0.0.1:8080")
}

#[derive(Parser)]
struct Opts {
    /// insert api url
    #[arg(
        short = 'u',
        long = "url",
        env = "API_URL",
        default_value = "https://victoria-monitoring.evan.ovh"
    )]
    url: String,

    /// API key
    #[arg(short = 'a', long = "apikey", env = "API_KEY")]
    apikey: String,

    /// exclude container name
    #[arg(short = 'e', long = "exclude", env = "EXCLUDE_CONTAINER")]
    exclude: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let opts: Opts = Opts::parse();

    let regex = Regex::new(&opts.exclude.clone().unwrap_or("".to_string())).unwrap();
    let docker = new_docker()?;
    let http_client = reqwest::Client::builder().build()?;
    let containers = get_container_list(&docker).await?;
    let mut tasks: Vec<JoinHandle<()>> = vec![];
    for container in containers {
        let http_clone = http_client.clone();
        let docker_clone = docker.clone();
        if (opts.exclude.is_some() && regex.is_match(&container.name)) {
            info!(
                "skipping container {} because of exclude filter.",
                container.id
            );
            continue;
        }

        tasks.push(tokio::spawn(async move {
            let _ = process_container(
                docker_clone,
                container,
                http_clone,
                opts.url.clone(),
                opts.apikey.clone(),
            )
            .await;
        }));
        break;
    }
    println!("finished spawning task.");
    for task in tasks {
        let _ = task.await;
    }
    println!("The end.");

    Ok(())
}
async fn process_container(
    docker: Docker,
    container: Container,
    http: Client,
    url: String,
    apikey: String,
) {
    println!("started process_container");
    //thread::sleep(time::Duration::from_millis(900));
    let timestamp = Instant::now();
    let mut old_value: Option<ContainerInfo> = None;
    let mut new_value: Option<ContainerInfo> = None;
    loop {
        let Some(some_stats) = docker.containers().get(&container.id).stats().next().await else {
            continue;
        };
        let Ok(stats) = some_stats else {
            continue;
        };

        let duration = Instant::now();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let Some(cpu_kernel) = stats.pointer("/cpu_stats/cpu_usage/usage_in_kernelmode") else {
            continue;
        };
        let Some(cpu_user) = stats.pointer("/cpu_stats/cpu_usage/usage_in_usermode") else {
            continue;
        };

        new_value = Some(ContainerInfo {
            cpu_kernel: cpu_kernel.as_f64().unwrap(),
            cpu_user: cpu_user.as_i64().unwrap(),
            timestamp: duration,
        });

        match old_value.clone() {
            None => {
                info!("old value is empty, probably initialising the thread.");
                old_value = new_value;
                continue;
            }
            Some(old) => {
                let mut hash = HashMap::new();
                hash.insert("__name__".to_string(), "cpu_kernel".to_string());
                hash.insert("job".to_string(), "rust_agent".to_string());
                let metric = VictoriaMetric {
                    metric: hash,
                    values: vec![new_value.unwrap().cpu_kernel.into()],
                    timestamps: vec![timestamp],
                };
                // let's send the new data diff to the server
                let url = format!("{}/insert", url);
                let body = serde_json::to_string(&metric).unwrap();
                trace!("sending data {:#?} at {}", body, timestamp);
                let req = http
                    .post(url)
                    .header("Authorization", format!("Bearer {}", apikey))
                    .header("Content-Type", "application/json")
                    .body(body);

                let err = req.send().await;
                if let Err(e) = err {
                    error!(
                        "could not access api, waiting 5 second before retrying :{:?}",
                        e
                    );
                    thread::sleep(time::Duration::from_secs(5));
                }
            }
        }
        // todo : compute ram real usage (usage less each cache)
        // let mem_usage = stats.pointer("/memory_stats/usage");
        // let mem_limit = stats.pointer("/memory_stats/limit");
        info!("got stat for id {:#?}: {}", container.id, cpu_kernel);

        // for now no cache system is set before sending data to the server, so there will be one request per container per second.
    }
}
#[derive(Debug, Clone)]
struct ContainerInfo {
    cpu_kernel: f64,
    cpu_user: i64,
    //ram_absolute: i64,
    //ram_percentage: i64,
    timestamp: Instant,
}
struct Container {
    id: String,
    image: String,
    state: ContainerStateStatusInlineItem,
    name: String,
}
async fn get_container_list(docker: &Docker) -> Result<Vec<Container>, Box<dyn std::error::Error>> {
    let opts = ContainerListOpts::builder().all(true).build();
    let items = docker.containers().list(&opts).await?;
    let mut total_count = 0;
    let mut valid_count = 0;
    let mut list: Vec<Container> = vec![];
    for container in items {
        total_count = total_count + 1;

        let state = match container.state.clone().unwrap().as_str() {
            "created" => ContainerStateStatusInlineItem::Created,
            "running" => ContainerStateStatusInlineItem::Running,
            "restarting" => ContainerStateStatusInlineItem::Restarting,
            "dead" => ContainerStateStatusInlineItem::Dead,
            "paused" => ContainerStateStatusInlineItem::Paused,
            "removing" => ContainerStateStatusInlineItem::Removing,
            _ => continue, // ignore exited container and non valid.
        };
        list.push(Container {
            id: container.id.unwrap_or_default()[..12].to_string(),
            image: container.image.unwrap_or_default(),
            state: state,
            name: container.names.map(|n| n[0].to_owned()).unwrap_or_default(),
        });
        valid_count = valid_count + 1;
    }
    debug!("{}/{} containers are valid", valid_count, total_count);

    return Ok(list);
}
