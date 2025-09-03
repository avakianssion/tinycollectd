//! Main module for tinyd.
use clap::{Parser, ValueEnum};
use serde_json::json;
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
    Service,
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
        let mut metrics: Vec<u8> = Vec::new(); // metrics vector to hold data to be sent

        // Little state machine action, which I'm sure there is a better, more idiomatic way of doing this.
        if cli.metrics.contains(&MetricType::All) {
            metrics = serde_json::to_vec(&collector::get_sysinfo(&sys)).unwrap();
        } else {
            if cli.metrics.contains(&MetricType::Service) {
                metrics.extend(
                    serde_json::to_vec(&collector::get_service_status(&cli.services)).unwrap(),
                );
            } else if cli.metrics.contains(&MetricType::DiskUsage) {
                metrics.extend(serde_json::to_vec(&collector::get_disk_usage()).unwrap());
            } else if cli.metrics.contains(&MetricType::Network) {
                metrics.extend(serde_json::to_vec(&collector::get_if_data()).unwrap());
            } else if cli.metrics.contains(&MetricType::Cpufreq) {
                metrics.extend(serde_json::to_vec(&collector::cpu_freq_json(&sys)).unwrap());
            } else if cli.metrics.contains(&MetricType::Uptime) {
                metrics.extend(serde_json::to_vec(&collector::uptime_json(&sys)).unwrap());
            }
        }
        // This feels like it should be in the collector module, but I don't see a clean way of getting it in there
        let combined = json!({
            "timestamp": &collector::get_timestamp(),
            "hostname": &collector::get_hostname(&sys),
            "metrics": metrics,
        });
        let bytes = serde_json::to_vec(&combined).unwrap();
        // Send UDP packet
        if let Err(e) = socket.send_to(&bytes, cli.destination).await {
            eprintln!("Failed to send UDP packet: {}", e);
        } else {
            println!(
                "Sent metrics to {} ({} metrics)",
                cli.destination,
                &metrics.len()
            );
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
