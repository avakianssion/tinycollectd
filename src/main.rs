//! Main module for tinycollectd.
use clap::{Parser, ValueEnum};
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
    /// metrics tinycollectd would collect
    #[arg(long, value_enum, value_delimiter = ',', default_value = "All")]
    metrics: Vec<MetricType>,
    /// interval for data to be collected in seconds.
    #[arg(long, default_value = "10")]
    collection_interval: u64,
}
#[derive(ValueEnum, Clone)]
enum MetricType {
    All,
    DiskUsage,
    Network,
    Cpufreq,
    Uptime,
    Service,
}
/// Function to add hostname, timestamp, and other metadata to individual metrics

/// Entrypoint for tinycollectd async runtime.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Create UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    // System object for collectors to share
    let mut sys = System::new_all();

    loop {
        sys.refresh_all(); // refresh once on every collection attempt
        let bytes = serde_json::to_vec(&collector::get_sysinfo(&sys)).unwrap();

        // Send UDP packet
        if let Err(e) = socket.send_to(&bytes, cli.destination).await {
            eprintln!("Failed to send UDP packet: {}", e);
        } else {
            println!(
                "Sent metrics to {} ({} bytes)",
                cli.destination,
                &bytes.len()
            );
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
