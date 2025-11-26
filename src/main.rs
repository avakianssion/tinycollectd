//! Main module for tinyd.
use clap::{Parser, ValueEnum};
use serde_json::{Value, json, Map};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use sysinfo::System;
use tinyd::collector;
use tokio::net::UdpSocket;
#[derive(Parser)]
struct Cli {
    /// destination for metrics (e.g. 127.0.0.1:1555)
    #[arg(long, default_value_t = SocketAddrV4::new(Ipv4Addr::new(127,0,0,1), 1555))]
    destination: SocketAddrV4,
    /// metrics tinyd would collect
    #[arg(long, value_enum, value_delimiter = ',', default_value = "All")]
    metrics: Vec<MetricType>,
    /// list of services to pull status
    #[arg(long)]
    services: Vec<String>,
    /// interval for data to be collected in seconds.
    #[arg(long, default_value = "10")]
    collection_interval: u64,
}
#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum MetricType {
    All,
    DiskUsage,
    Network,
    Cpufreq,
    Uptime,
    SmartLog,
}
/// Function to add hostname, timestamp, and other metadata to individual metrics

/// Entrypoint for tinyd async runtime.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let mut sys = System::new_all();

    loop {
        sys.refresh_all();

        let metrics_value = if cli.metrics.contains(&MetricType::All) {
            collector::get_sysinfo(&sys)
        } else {
            let mut metrics_obj = Map::new();

            if cli.metrics.contains(&MetricType::SmartLog) {
                let smart_log = collector::collect_smart_log(); // Vec<NvmesSmartLog>
                let smart_val = serde_json::to_value(smart_log).unwrap_or_else(|_| json!([]));
                metrics_obj.insert("smart_log".to_string(), smart_val);
            }

            if cli.metrics.contains(&MetricType::DiskUsage) {
                let disk_data = collector::get_disk_usage(); // Vec<Value>
                metrics_obj.insert("disk_usage".to_string(), Value::Array(disk_data));
            }

            if cli.metrics.contains(&MetricType::Network) {
                let network_data = collector::get_if_data(); // Vec<Value>
                metrics_obj.insert("network".to_string(), Value::Array(network_data));
            }

            if cli.metrics.contains(&MetricType::Cpufreq) {
                let cpu_data = collector::cpu_freq_json(&sys);
                if let Value::Object(map) = cpu_data {
                    metrics_obj.insert("cpufreq".to_string(), Value::Object(map));
                }
            }

            if cli.metrics.contains(&MetricType::Uptime) {
                let uptime_data = collector::uptime_json(&sys);
                if let Value::Object(map) = uptime_data {
                    metrics_obj.insert("uptime".to_string(), Value::Object(map));
                }
            }

            Value::Object(metrics_obj)
        };

        let combined = json!({
            "timestamp": collector::get_timestamp(),
            "hostname": collector::get_hostname(&sys),
            "metrics": metrics_value,
        });

        let bytes = serde_json::to_vec(&combined).unwrap();

        if let Err(e) = socket.send_to(&bytes, cli.destination).await {
            eprintln!("Failed to send UDP packet: {}", e);
        } else {
            println!(
                "Sent metrics to {} ({} bytes)",
                cli.destination,
                bytes.len()
            );
        }

        tokio::time::sleep(Duration::from_secs(cli.collection_interval)).await;
    }
}