//! Main module for tinycollectd.
mod collector;
use clap::{Parser, ValueEnum};
use std::time::Duration;
use sysinfo::System;
use tokio::net::UdpSocket;
#[derive(Parser)]
struct Cli {
    /// send_host to send metrics to
    #[arg(long, default_value = "127.0.0.1")]
    send_host: Option<String>,
    /// send_port to send metrics to
    #[arg(long, default_value = "1555")]
    send_port: Option<u16>,
    /// metrics tinyd would collectd
    #[arg(long, value_enum, value_delimiter = ',', default_value = "All")]
    metrics: Vec<MetricType>,
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

/// Entrypoint for tinycollectd async runtime.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    if let Some(send_host) = cli.send_host.as_deref() {
        println!("Sending host: {}", send_host);
    }
    if let Some(send_port) = cli.send_port {
        println!("Sending port: {}", send_port);
    }
    // Get target from environment or use default
    let target = std::env::var("METRICS_TARGET")
        .unwrap_or_else(|_| format!("{:?}:{:?}", cli.send_host, cli.send_port));

    println!("Sending metrics to UDP {}", target);

    // Create UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    loop {
        // TODO: here, we should be calling the relevant functions in collector based on what was
        // in the cli
        let bytes = serde_json::to_vec(&collector::get_sysinfo(System::new_all())).unwrap();

        // Send UDP packet
        if let Err(e) = socket.send_to(&bytes, &target).await {
            eprintln!("Failed to send UDP packet: {}", e);
        } else {
            println!("Sent metrics to {} ({} bytes)", target, &bytes.len());
        }

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
