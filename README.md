# tinycollectd

A lightweight Rust-based tool to collect system metrics (CPU, uptime, disk usage, network stats, and more...) and send them as JSON over UDP at regular intervals.

This project was **inspired by [collectd](https://github.com/collectd/collectd)** — and born out of some frustrations while using it.  
The goal is to provide a simpler, smaller, and more modern alternative that’s easy? easier to configure and extend, or maybe not and this would be another terrible metrics collector that people hate using.

## Inspiration
Most metrics and monitoring tools suffer from unnecessary complexity. Engineers spend more time managing configuration files than collecting meaningful data.
The current ecosystem is plagued by overengineering. Most legacy codebases are unmaintained, while memory-intensive implementations (particularly those built on interpreted languages with heavy codec layers) consume resources disproportionate to their functionality.

## Tenets
These are the principles we follow to build better tooling:
1. **Simplicity** – One job. Done well. No bloat.
2. **Maturity** – Grow slow, get stable.
3. **Minimal Config** – Flags over `.conf` whenever possible. Less editing, more running.
4. **Minimal Use of Plugins** – Plugins are where tools go to die. If we want plugins, this has to be added very very carefully.
