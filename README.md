# tinycollectd

A lightweight Rust-based tool to collect system metrics (CPU, uptime, disk usage, network stats, and more...) and send them as JSON over UDP at regular intervals.

This project was **inspired by [collectd](https://github.com/collectd/collectd)** — and born out of some frustrations while using it.  
The goal is to provide a simpler, smaller, and more modern alternative that’s easy? easier to configure and extend, or maybe not and this would be another terrible metrics collector that people hate using.


## Tenets
These are the principles we follow to build better tooling:
1. **Simplicity** – One job. Done well. No bloat.
2. **Maturity** – Grow slow, get stable.
3. **Minimal Config** – Flags over `.conf` whenever possible. Less editing, more running.
4. **Minimal Use of Plugins** – Plugins are hard to manage. They have to be added very very carefully, if at all.

## Usage

```bash
Usage: tinyd [OPTIONS]

Options:
      --send-host <SEND_HOST>
          send_host to send metrics to [default: 127.0.0.1]

      --send-port <SEND_PORT>
          send_port to send metrics to [default: 1555]

      --metrics <METRICS>
          metrics tinycollectd will collect [default: all]
          [possible values: all, disk-usage, network, cpufreq, uptime, service]

      --collection-interval <COLLECTION_INTERVAL>
          interval (seconds) for data collection [default: 10]

  -h, --help
          Print help
