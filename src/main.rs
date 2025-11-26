//! Main module for tinyd.
use clap::{Parser, ValueEnum};
use serde_json::{Value, json};
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

    // Create UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    // System object for collectors to share
    let mut sys = System::new_all();

    loop {
        sys.refresh_all(); // refresh once on every collection attempt

        let metrics_value = if cli.metrics.contains(&MetricType::All) {
            collector::get_sysinfo(&sys)
        } else {
            let mut combined_object = serde_json::Map::new();
            let mut combined_arrays = Vec::new();

            if cli.metrics.contains(&MetricType::SmartLog) {
                let smart_log = collector::collect_smart_log();
                combined_arrays.extend(smart_log);
            }

            if cli.metrics.contains(&MetricType::DiskUsage) {
                let disk_data = collector::get_disk_usage();
                combined_arrays.extend(disk_data);
            }

            if cli.metrics.contains(&MetricType::Network) {
                let network_data = collector::get_if_data();
                combined_arrays.extend(network_data);
            }

            if cli.metrics.contains(&MetricType::Cpufreq) {
                let cpu_data = collector::cpu_freq_json(&sys);
                if let Value::Object(map) = cpu_data {
                    combined_object.extend(map);
                }
            }

            if cli.metrics.contains(&MetricType::Uptime) {
                let uptime_data = collector::uptime_json(&sys);
                if let Value::Object(map) = uptime_data {
                    combined_object.extend(map);
                }
            }

            // Combine single values and arrays
            if !combined_object.is_empty() && !combined_arrays.is_empty() {
                combined_object.insert("array_data".to_string(), Value::Array(combined_arrays));
                Value::Object(combined_object)
            } else if !combined_object.is_empty() {
                Value::Object(combined_object)
            } else if !combined_arrays.is_empty() {
                Value::Array(combined_arrays)
            } else {
                json!({})
            }
        };

        let combined = json!({
            "timestamp": collector::get_timestamp(),
            "hostname": collector::get_hostname(&sys),
            "metrics": metrics_value,
        });

        let bytes = serde_json::to_vec(&combined).unwrap();

        // Send UDP packet
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