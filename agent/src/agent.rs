use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[cfg(unix)]
use anyhow::Result;
use docker_api::{Docker, opts::ContainerListOpts};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
};
use tracing::{error, info, warn};

use crate::{
    Opts,
    container::{Container, ContainerStats},
};

#[cfg(unix)]
pub fn new_docker() -> Result<Docker> {
    Ok(Docker::unix("/var/run/docker.sock"))
}

#[cfg(not(unix))]
pub fn new_docker() -> Result<Docker> {
    Docker::new("tcp://127.0.0.1:8080")
}

pub struct Agent {
    opts: Opts,
    docker: Docker,
    client: Client,

    is_shutting_down: AtomicBool,

    refresh_thread: RwLock<Option<JoinHandle<()>>>,

    containers: RwLock<HashMap<String, Arc<RwLock<Container>>>>,
    container_processors: RwLock<HashMap<String, JoinHandle<()>>>,

    error_lock: Mutex<()>,
}

impl Agent {
    pub async fn start(opts: Opts) -> Result<Arc<Self>> {
        let docker = self::new_docker()?;
        let client = reqwest::Client::builder().build()?;

        let agent = Arc::new(Self {
            opts,
            docker,
            client,

            is_shutting_down: AtomicBool::new(false),

            refresh_thread: RwLock::new(None),

            containers: RwLock::new(HashMap::new()),
            container_processors: RwLock::new(HashMap::new()),

            error_lock: Mutex::new(()),
        });

        agent.refresh_containers().await?;

        let agent_clone = Arc::clone(&agent);
        agent
            .refresh_thread
            .write()
            .await
            .replace(tokio::spawn(async move {
                agent_clone.automatic_refresh().await;
            }));

        Ok(agent)
    }
    pub fn is_excluded(self: &Arc<Self>, container: &Container) -> bool {
        match self.opts.exclude.clone() {
            None => {
                return false;
            }
            Some(filter) => {
                let re = Regex::new(&filter);
                match re {
                    Err(_) => {
                        return false;
                    }
                    Ok(regex) => {
                        if regex.is_match(&container.state().to_string().to_lowercase()) {
                            return true;
                        }
                    }
                }
                return false;
            }
        }
    }
    pub async fn refresh_containers(self: &Arc<Self>) -> Result<()> {
        let opts = ContainerListOpts::builder().all(true).build();
        let containers = self.docker.containers().list(&opts).await?;
        let container_ids = containers
            .into_iter()
            .filter_map(|summary| summary.id)
            .collect::<Vec<_>>();

        // Find the containers to remove.
        let containers_to_remove;
        let containers_to_add;

        {
            let containers_lock = self.containers.read().await;

            containers_to_remove = containers_lock
                .keys()
                .filter(|id| !container_ids.contains(id))
                .cloned()
                .collect::<Vec<_>>();

            // Find the containers to add.
            containers_to_add = container_ids
                .into_iter()
                .filter(|id| !containers_lock.contains_key(id))
                .collect::<Vec<_>>();
        }

        // Remove containers.
        {
            let mut containers_lock = self.containers.write().await;
            let mut processors_lock = self.container_processors.write().await;

            for id in containers_to_remove {
                let container = containers_lock.remove(&id);
                if let Some(container) = container {
                    let container_lock = container.read().await;

                    info!(
                        "Removed container \"{}\" [{}]",
                        container_lock.name(),
                        container_lock.id()
                    );
                }

                // Stop the processor.
                if let Some(handle) = processors_lock.remove(&id) {
                    handle.abort();
                }
            }
        }

        // Add containers.
        for id in containers_to_add {
            let container = Container::new(&self.docker, &id).await?;

            if (self.is_excluded(&container)) {
                continue;
            }
            info!(
                "Added container \"{}\" [{}]",
                container.name(),
                container.id()
            );

            let container_id = container.id().to_string();

            let container_rc = Arc::new(RwLock::new(container));
            self.containers
                .write()
                .await
                .insert(container_id.clone(), Arc::clone(&container_rc));

            let agent_clone = Arc::clone(&self);
            let container_clone = Arc::clone(&container_rc);

            // Spawn a task to process the container.
            let handle = tokio::spawn(async move {
                let _ = agent_clone.process_container(container_clone).await;
            });

            self.container_processors
                .write()
                .await
                .insert(container_id.clone(), handle);
        }

        Ok(())
    }

    async fn automatic_refresh(self: Arc<Self>) {
        let refresh_interval = std::time::Duration::from_secs(3);

        loop {
            tokio::time::sleep(refresh_interval).await;
            if let Err(e) = self.refresh_containers().await {
                warn!("Error refreshing containers: {}", e);
            }
        }
    }

    async fn process_container(&self, container: Arc<RwLock<Container>>) -> Result<()> {
        let mut previous_stats = None;

        while !self.is_shutting_down.load(Ordering::SeqCst) {
            {
                let container_lock = container.read().await;
                let stats = container_lock.query_stats().await?;

                if let Some(previous_stats) = &previous_stats {
                    self.upload_stats(&container_lock, &stats, &previous_stats)
                        .await?;
                }

                previous_stats.replace(stats);
            }

            {
                let mut container_lock = container.write().await;
                container_lock.reinspect().await?;
            }
        }

        Ok(())
    }

    async fn upload_stats(
        &self,
        container: &Container,
        stats: &ContainerStats,
        previous: &ContainerStats,
    ) -> Result<()> {
        let timestamp = stats.unix_timestamp as i64;

        let online_cpus = stats.online_cpus as f64;

        let system_cpu = (stats.system_cpu_usage - previous.system_cpu_usage) as f64;
        let total_cpu = (stats.total_cpu_usage - previous.total_cpu_usage) as f64;
        let kernelmode_cpu = (stats.kernelmode_cpu_usage - previous.kernelmode_cpu_usage) as f64;
        let usermode_cpu = (stats.usermode_cpu_usage - previous.usermode_cpu_usage) as f64;
        let cpu_usage_percent = stats.cpu_usage_percent(previous);

        let memory_usage = stats.memory_usage_bytes as f64;
        let memory_limit = stats.memory_limit_bytes as f64;
        let memory_percent = stats.memory_usage_percent();

        let network_rx_bytes = (stats.network_rx_bytes - previous.network_rx_bytes) as f64;
        let network_tx_bytes = (stats.network_tx_bytes - previous.network_tx_bytes) as f64;

        let futures = [
            self.upload_stat(container, "cpu_online_cpus", online_cpus, timestamp),
            self.upload_stat(container, "cpu_system_usage", system_cpu, timestamp),
            self.upload_stat(container, "cpu_total_usage", total_cpu, timestamp),
            self.upload_stat(container, "cpu_kernelmode_usage", kernelmode_cpu, timestamp),
            self.upload_stat(container, "cpu_usermode_usage", usermode_cpu, timestamp),
            self.upload_stat(container, "cpu_usage_percent", cpu_usage_percent, timestamp),
            self.upload_stat(container, "memory_usage_bytes", memory_usage, timestamp),
            self.upload_stat(container, "memory_limit_bytes", memory_limit, timestamp),
            self.upload_stat(container, "memory_usage_percent", memory_percent, timestamp),
            self.upload_stat(container, "network_rx_bytes", network_rx_bytes, timestamp),
            self.upload_stat(container, "network_tx_bytes", network_tx_bytes, timestamp),
        ];

        futures::future::join_all(futures).await;

        Ok(())
    }

    async fn upload_stat(
        &self,
        container: &Container,
        stat_name: &str,
        stat_value: f64,
        timestamp: i64,
    ) -> Result<()> {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct VictoriaMetric {
            pub metric: HashMap<String, String>,
            pub values: Vec<f64>,
            pub timestamps: Vec<i64>,
        }

        let mut hash = HashMap::new();
        hash.insert("__name__".to_string(), stat_name.to_string());
        hash.insert("container_name".to_string(), container.name().to_string());

        let metric = VictoriaMetric {
            metric: hash,
            values: vec![stat_value],
            timestamps: vec![timestamp],
        };

        // let's send the new data diff to the server
        let url = format!("{}/insert", self.opts.url);
        let body = serde_json::to_string(&metric).unwrap();

        let req = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.opts.apikey))
            .header("Content-Type", "application/json")
            .body(body);

        let err = req.send().await;
        if let Err(e) = err {
            error!(
                "Could not access API, waiting 5 second before retrying :{:?}",
                e
            );

            let _lock = self.error_lock.lock().await;
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }

    pub async fn shutdown(&self) {
        self.is_shutting_down.store(true, Ordering::SeqCst);

        let mut handles = Vec::new();

        {
            let mut refresh_thread_lock = self.refresh_thread.write().await;
            if let Some(handle) = refresh_thread_lock.take() {
                handles.push(handle);
            }
        }

        {
            let mut processors_lock = self.container_processors.write().await;
            for (_, handle) in processors_lock.drain() {
                handles.push(handle);
            }
        }

        error!("Shutting down, waiting for tasks to finish...");

        for handle in &handles {
            handle.abort();
        }

        futures::future::join_all(handles).await;
    }
}
