use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow};
use docker_api::{
    Docker,
    models::{ContainerInspect200Response, ContainerStateStatusInlineItem},
};
use futures::StreamExt;

pub struct Container {
    docker_container: docker_api::Container,

    status: ContainerStatus,
}

impl Container {
    pub async fn new(docker: &Docker, id: &String) -> Result<Self> {
        let docker_container = docker.containers().get(id);
        let status = ContainerStatus::new(&docker_container).await?;

        Ok(Self {
            docker_container,
            status,
        })
    }

    pub async fn reinspect(&mut self) -> Result<()> {
        self.status = ContainerStatus::new(&self.docker_container).await?;

        Ok(())
    }

    pub async fn query_stats(&self) -> Result<ContainerStats> {
        ContainerStats::query(&self.docker_container).await
    }

    pub fn id(&self) -> &str {
        self.docker_container.id().as_ref()
    }

    pub fn image(&self) -> &str {
        &self.status.image
    }

    pub fn name(&self) -> &str {
        &self.status.name
    }

    pub fn state(&self) -> &ContainerStateStatusInlineItem {
        &self.status.state
    }
}

#[derive(Debug)]
pub struct ContainerStatus {
    inspection_time: Instant,
    inspection: ContainerInspect200Response,

    image: String,
    name: String,
    state: ContainerStateStatusInlineItem,
}

impl ContainerStatus {
    async fn new(docker_container: &docker_api::Container) -> Result<Self> {
        let inspection = docker_container.inspect().await?;
        let inspection_time = Instant::now();

        let image = inspection
            .config
            .as_ref()
            .and_then(|config| config.image.clone())
            .ok_or_else(|| anyhow!("Failed to get container image"))?;

        let name = inspection
            .name
            .as_ref()
            .map(|n| n.trim_start_matches('/').to_string())
            .ok_or_else(|| anyhow!("Failed to get container name"))?;

        let state_str = inspection
            .state
            .as_ref()
            .and_then(|state| state.status.clone())
            .ok_or_else(|| anyhow!("Failed to get container state"))?;

        let state = match state_str.as_str() {
            "created" => ContainerStateStatusInlineItem::Created,
            "running" => ContainerStateStatusInlineItem::Running,
            "paused" => ContainerStateStatusInlineItem::Paused,
            "restarting" => ContainerStateStatusInlineItem::Restarting,
            "removing" => ContainerStateStatusInlineItem::Removing,
            "exited" => ContainerStateStatusInlineItem::Exited,
            "dead" => ContainerStateStatusInlineItem::Dead,
            _ => return Err(anyhow!("Unknown container state: {}", state_str)),
        };

        Ok(Self {
            inspection_time,
            inspection,

            image,
            name,
            state,
        })
    }
}

#[derive(Debug)]
pub struct ContainerStats {
    pub capture_time: Instant,
    pub unix_timestamp: u64,

    pub online_cpus: u64,
    pub system_cpu_usage: u64,
    pub total_cpu_usage: u64,
    pub kernelmode_cpu_usage: u64,
    pub usermode_cpu_usage: u64,

    pub memory_usage: u64,
    pub memory_limit: u64,

    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

impl ContainerStats {
    pub async fn query(docker_container: &docker_api::Container) -> Result<Self> {
        let stats = loop {
            if let Some(Ok(stats)) = docker_container.stats().next().await {
                break stats;
            }
        };

        let capture_time = Instant::now();
        let unix_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let online_cpus = stats
            .pointer("/cpu_stats/online_cpus")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let system_cpu_usage = stats
            .pointer("/cpu_stats/system_cpu_usage")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let total_cpu_usage = stats
            .pointer("/cpu_stats/cpu_usage/total_usage")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let kernelmode_cpu_usage = stats
            .pointer("/cpu_stats/cpu_usage/usage_in_kernelmode")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let usermode_cpu_usage = stats
            .pointer("/cpu_stats/cpu_usage/usage_in_usermode")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let memory_usage = stats
            .pointer("/memory_stats/usage")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let memory_limit = stats
            .pointer("/memory_stats/limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Iterate over all networks to sum rx and tx bytes.
        let mut network_rx_bytes = 0;
        let mut network_tx_bytes = 0;

        if let Some(networks) = stats.pointer("/networks").and_then(|v| v.as_object()) {
            for (_, net_stats) in networks {
                if let Some(rx_bytes) = net_stats.get("rx_bytes").and_then(|v| v.as_u64()) {
                    network_rx_bytes += rx_bytes;
                }
                if let Some(tx_bytes) = net_stats.get("tx_bytes").and_then(|v| v.as_u64()) {
                    network_tx_bytes += tx_bytes;
                }
            }
        }

        Ok(Self {
            capture_time,
            unix_timestamp,

            online_cpus,
            system_cpu_usage,
            total_cpu_usage,
            kernelmode_cpu_usage,
            usermode_cpu_usage,

            memory_usage,
            memory_limit,

            network_rx_bytes,
            network_tx_bytes,
        })
    }
}
