// src/collector/services.rs
//! systemd service status collection.

use serde_json::{Value, json};
use std::process::Command;
use std::str;

/// Function to extract status of a list of services.
pub fn get_service_status(services: &[String]) -> Vec<Value> {
    let mut results = Vec::new();

    for service in services {
        let status = get_service_active_status(service);

        results.push(json!({
            "service_name": service,
            "status": status
        }));
    }

    results
}

/// Function to check whether a service is active or not.
fn get_service_active_status(service: &str) -> String {
    match Command::new("systemctl")
        .args(["is-active", service])
        .output()
    {
        Ok(output) => str::from_utf8(&output.stdout)
            .unwrap_or("unknown")
            .trim()
            .to_string(),
        Err(_) => "error".to_string(),
    }
}
