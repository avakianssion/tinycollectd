// src/main.rs
mod collector;
use std::time::Duration;
use sysinfo::System;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get target from environment or use default
    let target = std::env::var("METRICS_TARGET").unwrap_or_else(|_| "127.0.0.1:1555".to_string());

    println!("Sending metrics to UDP {}", target);

    // Create UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    loop {
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
