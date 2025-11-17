#![allow(clippy::suspicious_else_formatting)]
mod common;
use clap::Parser;
use common::{new_docker, print_chunk};
use docker_api::Docker;
use docker_api::models::{ContainerState, ContainerStateStatusInlineItem};
use docker_api::opts::ContainerListOpts;
use futures::StreamExt;
use std::path::PathBuf;

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
    println!("what ?");
    let _ = get_container_list(docker).await?;
    println!("nani ?");
    Ok(())
}
struct container {
    id: String,
    image: String,
    state: ContainerStateStatusInlineItem,
    name: String,
}
async fn get_container_list(docker: Docker) -> Result<Vec<container>, Box<dyn std::error::Error>> {
    println!("what ?");
    let opts = ContainerListOpts::builder().all(true).build();
    let mut list: Vec<container> = vec![];
    match docker.containers().list(&opts).await {
        Err(e) => eprintln!("Error: {e}"),
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
    }

    return Err("not implemented".into());
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
