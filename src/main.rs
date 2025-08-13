// src/main.rs
use sysinfo::{System, Disks, Networks};
use std::time::Duration;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sys = System::new_all();
    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();
    
    // Get target from environment or use default
    let target = std::env::var("METRICS_TARGET")
        .unwrap_or_else(|_| "127.0.0.1:1555".to_string());
    
    println!("Sending metrics to UDP {}", target);
    
    // Create UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    
    loop {
        sys.refresh_all();
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs();
        
        let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
        let uptime = System::uptime();
        let cpu_freq = sys.cpus().first().map(|cpu| cpu.frequency()).unwrap_or(0);
        
        // Build disk usage JSON manually
        let mut disk_json = String::from("[");
        for (i, disk) in disks.iter().enumerate() {
            if i > 0 { disk_json.push(','); }
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let used_percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
            
            disk_json.push_str(&format!(
                r#"{{"mount":"{}","total_gb":{},"used_gb":{},"used_percent":{:.1}}}"#,
                disk.mount_point().to_string_lossy().replace('"', "\\\""),
                total / 1_000_000_000,
                used / 1_000_000_000,
                used_percent
            ));
        }
        disk_json.push(']');
        
        // Build network JSON manually
        let mut network_json = String::from("[");
        for (i, (name, data)) in networks.iter().enumerate() {
            if i > 0 { network_json.push(','); }
            network_json.push_str(&format!(
                r#"{{"interface":"{}","rx_bytes":{},"tx_bytes":{}}}"#,
                name.replace('"', "\\\""),
                data.total_received(),
                data.total_transmitted()
            ));
        }
        network_json.push(']');
        
        // Build complete JSON
        let json = format!(
            r#"{{"timestamp":{},"hostname":"{}","uptime":{},"cpu_freq_mhz":{},"disk_usage":{},"network":{}}}"#,
            timestamp,
            hostname.replace('"', "\\\""),
            uptime,
            cpu_freq,
            disk_json,
            network_json
        );
        
        // Send UDP packet
        if let Err(e) = socket.send_to(json.as_bytes(), &target).await {
            eprintln!("Failed to send UDP packet: {}", e);
        } else {
            println!("Sent metrics to {} ({} bytes)", target, json.len());
        }
        
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}