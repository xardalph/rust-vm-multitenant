#![allow(clippy::suspicious_else_formatting)]

use ::time::Duration;
use axum_login::tracing::{debug, info};
use clap::Parser;
use docker_api::Docker;
use docker_api::models::{ContainerState, ContainerStateStatusInlineItem};
use docker_api::opts::ContainerListOpts;
use futures::{Stream, StreamExt};
use nosqlensiie::nosql::model::VictoriaMetric;
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
    #[command(subcommand)]
    subcmd: Cmd,
}

#[derive(Parser)]
enum Cmd {
    /// Returns usage statistics of the container.
    Stats { id: String },
    Logs {
        id: String,
        #[arg(long)]
        stdout: bool,
        #[arg(long)]
        stderr: bool,
    },
    List {
        #[arg(long)]
        all: bool,
    },
    /// Returns information about running processes in the container.
    Top {
        id: String,
        /// Arguments passed to `ps` in the container.
        psargs: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let opts: Opts = Opts::parse();
    let docker = new_docker()?;
    let http_client = reqwest::Client::builder().build()?;
    let containers = get_container_list(&docker).await?;
    let mut tasks: Vec<JoinHandle<()>> = vec![];
    for container in containers {
        let http_clone = http_client.clone();
        let docker_clone = docker.clone();

        tasks.push(tokio::spawn(async move {
            let _ = process_container(docker_clone, container, http_clone).await;
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
async fn process_container(docker: Docker, container: Container, http: Client) {
    println!("started process_container");
    //thread::sleep(time::Duration::from_millis(900));
    let timestamp = Instant::now();
    let mut old_value: Option<ContainerInfo> = None;
    let mut new_value: Option<ContainerInfo> = None;
    for i in (1..30) {
        println!("i is : {i}");
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
        // to manage unwrap we should take current timestamp, and maybe use it for duration compute ?
        let _ = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        //println!("{stats:#?}");
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

        let old_value_clone = old_value.clone();
        match old_value_clone {
            None => {
                info!("old value is empty, probably initialising the thread.");
                old_value = new_value;
                continue;
            }
            Some(old) => {
                let mut hash = HashMap::new();
                hash.insert("__name__".to_string(), "cpu_kernel".to_string());
                let metric = VictoriaMetric {
                    metric: hash,
                    values: vec![new_value.unwrap().cpu_kernel.into()],
                    timestamps: vec![timestamp],
                };
                // let's send the new data diff to the server
                let url = "http://localhost:3000/insert";
                let body = serde_json::to_string(&metric).unwrap();
                println!("sending data {:#?} at {}", body, timestamp);
                let req = http
                    .post(url)
                    .header("Authorization", "Bearer secrettokenreversible")
                    .header("Content-Type", "application/json")
                    .body(body);
                println!("{:#?}", req);
                let err = req.send().await;
                println!("response : {:?}", err);
            }
        }
        // todo : compute ram real usage (usage less each cache)
        // let mem_usage = stats.pointer("/memory_stats/usage");
        // let mem_limit = stats.pointer("/memory_stats/limit");
        info!("got stat for id {:#?}", container.id);
        info!("final value :  {:?}", cpu_kernel);
        // for now no cache system is set before sending data to the server, so there will be one request per container per second
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
        debug!("{}", state);
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
/*
match opts.subcmd {
    Cmd::List { all } => {
        use docker_api::opts::ContainerListOpts;

        let opts = if all {
            ContainerListOpts::builder().all(true).build()
        } else {
            Default::default()
        };
        match docker.containers().list(&opts).await {
            Ok(containers) => {
                containers.into_iter().for_each(|container| {
                    println!(
                        "{}\t{}\t{:?}\t{}\t{}",
                        &container.id.unwrap_or_default()[..12],
                        container.image.unwrap_or_default(),
                        container.state,
                        container.status.unwrap_or_default(),
                        container.names.map(|n| n[0].to_owned()).unwrap_or_default()
                    );
                });
            }
            Err(e) => eprintln!("Error: {e}"),
        }
    }
    Cmd::Logs { id, stdout, stderr } => {
        use docker_api::opts::LogsOpts;
        let container = docker.containers().get(&id);
        let logs_stream =
            container.logs(&LogsOpts::builder().stdout(stdout).stderr(stderr).build());

        let logs: Vec<_> = logs_stream
            .map(|chunk| match chunk {
                Ok(chunk) => chunk.to_vec(),
                Err(e) => {
                    eprintln!("Error: {e}");
                    vec![]
                }
            })
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        print!("{}", String::from_utf8_lossy(&logs));
    }
    Cmd::Stats { id } => {
        while let Some(result) = docker.containers().get(&id).stats().next().await {
            match result {
                Ok(stat) => println!("{stat:#?}"),
                Err(e) => eprintln!("Error: {e}"),
            }
        }
    }
    Cmd::Top { id, psargs } => {
        match docker.containers().get(&id).top(psargs.as_deref()).await {
            Ok(top) => println!("{top:#?}"),
            Err(e) => eprintln!("Error: {e}"),
        };
    }
}

 */
