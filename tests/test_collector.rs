use tinyd::collector::*;
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use sysinfo::System;

    /// Helper function to create a system instance for testing
    fn create_test_system() -> System {
        let mut sys = System::new();
        sys.refresh_all();
        sys
    }

    #[test]
    fn test_uptime_json() {
        let sys = create_test_system();
        let uptime_json = uptime_json(&sys);

        assert!(uptime_json.is_object());
        assert!(uptime_json["uptime"].is_string());

        let uptime_str = uptime_json["uptime"].as_str().unwrap();
        assert!(uptime_str.parse::<u64>().is_ok());
    }

    #[test]
    fn test_cpu_freq_json() {
        let sys = create_test_system();
        let freq_json = cpu_freq_json(&sys);

        assert!(freq_json.is_object());
        assert!(freq_json["cpu_freq_mhz"].is_string());

        let freq_str = freq_json["cpu_freq_mhz"].as_str().unwrap();
        assert!(freq_str.parse::<u64>().is_ok());
    }

    #[test]
    fn test_get_if_data() {
        let interfaces = get_if_data();

        assert!(interfaces.is_empty() || interfaces.len() > 0);

        for interface in &interfaces {
            assert!(interface.is_object());
            assert!(interface["interface"].is_string());
            assert!(interface["rx_bytes"].is_u64());
            assert!(interface["tx_bytes"].is_u64());

            let name = interface["interface"].as_str().unwrap();
            assert!(!name.is_empty());
        }
    }

    #[test]
    fn test_get_disk_usage() {
        let disks = get_disk_usage();

        assert!(disks.len() >= 0);

        for disk in &disks {
            assert!(disk.is_object());
            assert!(disk["mount"].is_string());
            assert!(disk["total_gb"].is_u64());
            assert!(disk["used_gb"].is_u64());
            assert!(disk["used_percent"].is_f64());

            // Mount point should not be empty
            let mount = disk["mount"].as_str().unwrap();
            assert!(!mount.is_empty());

            let total = disk["total_gb"].as_u64().unwrap();
            let used = disk["used_gb"].as_u64().unwrap();
            assert!(used <= total);

            let used_percent = disk["used_percent"].as_f64().unwrap();
            assert!(used_percent >= 0.0 && used_percent <= 100.0);
        }
    }

    #[test]
    fn test_get_sysinfo() {
        let sys = create_test_system();
        let sysinfo = get_sysinfo(&sys);

        assert!(sysinfo.is_object());
        assert!(sysinfo["timestamp"].is_u64());
        assert!(sysinfo["hostname"].is_string());
        assert!(sysinfo["uptime"].is_string());
        assert!(sysinfo["cpu_freq_mhz"].is_string());
        assert!(sysinfo["disk_usage"].is_array());
        assert!(sysinfo["network"].is_array());

        let timestamp = sysinfo["timestamp"].as_u64().unwrap();
        assert!(timestamp > 1_577_836_800, "Timestamp should be after 2020");

        let hostname = sysinfo["hostname"].as_str().unwrap();
        assert!(!hostname.is_empty());

        let uptime = sysinfo["uptime"].as_str().unwrap();
        assert!(uptime.parse::<u64>().is_ok());

        let cpu_freq = sysinfo["cpu_freq_mhz"].as_str().unwrap();
        assert!(cpu_freq.parse::<u64>().is_ok());
    }

    #[test]
    fn test_json_escaping() {
        // Test that quote escaping works correctly in the get_sysinfo function
        // This is more of an integration test since we can't easily mock the hostname but like who cares
        let sys = create_test_system();
        let sysinfo = get_sysinfo(&sys);

        let hostname = sysinfo["hostname"].as_str().unwrap();
        // Should not contain unescaped quotes
        assert!(!hostname.contains("\"") || hostname.contains("\\\""));
    }

    #[test]
    fn test_network_data_types() {
        let interfaces = get_if_data();

        for interface in &interfaces {
            let rx_bytes = interface["rx_bytes"].as_u64().unwrap();
            let tx_bytes = interface["tx_bytes"].as_u64().unwrap();

            // This test feels pointless cause u64 guarantees non-negative but doesn't hurt I guess\
            assert!(rx_bytes >= 0);
            assert!(tx_bytes >= 0);
        }
    }

    // Benchmark-style performance check
    #[test]
    fn test_performance() {
        let start = std::time::Instant::now();
        let sys = create_test_system();
        let _sysinfo = get_sysinfo(&sys);
        let duration = start.elapsed();

        // 1 second to gather all the data? this needs to be experimented with because
        // what happens when we are running on beefy servers.
        assert!(
            duration.as_secs() < 1,
            "System info collection should be fast"
        );
    }
}
