use serde::{Deserialize, Serialize};

use crate::wsl::execute_wsl;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitorData {
    pub wsl_version: String,
    pub distros: Vec<DistroMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroMetrics {
    pub name: String,
    pub memory_mb: u64,
    pub disk_usage: String,
    pub process_count: usize,
}

pub async fn collect_monitor_data() -> String {
    let mut data = MonitorData::default();

    match execute_wsl(&["--version"]).await {
        Ok(output) => {
            data.wsl_version = output.lines().next().unwrap_or("Unknown").trim().to_string();
        }
        Err(_) => {
            data.wsl_version = "Unknown".to_string();
        }
    }

    let distro_list = crate::wsl::refresh_distros().await;
    let distros = crate::wsl::parse_distros(&distro_list);

    for distro in &distros {
        let mut metrics = DistroMetrics {
            name: distro.name.clone(),
            memory_mb: 0,
            disk_usage: String::new(),
            process_count: 0,
        };

        if distro.state == "Running" {
            if let Ok(output) = execute_wsl(&[
                "-d",
                &distro.name,
                "--",
                "bash",
                "-c",
                "ps aux --no-headers 2>/dev/null | wc -l",
            ])
            .await
            {
                metrics.process_count = output.trim().parse().unwrap_or(0);
            }

            if let Ok(output) = execute_wsl(&[
                "-d",
                &distro.name,
                "--",
                "bash",
                "-c",
                "df -h / 2>/dev/null | tail -1 | awk '{print $3\"/\"$2\" (\"$5\")\"}'",
            ])
            .await
            {
                metrics.disk_usage = output.trim().to_string();
            }

            if let Ok(output) = execute_wsl(&[
                "-d",
                &distro.name,
                "--",
                "bash",
                "-c",
                "free -m 2>/dev/null | awk '/^Mem:/{print $3}'",
            ])
            .await
            {
                metrics.memory_mb = output.trim().parse().unwrap_or(0);
            }
        }

        data.distros.push(metrics);
    }

    serde_json::to_string(&data).unwrap_or_default()
}
