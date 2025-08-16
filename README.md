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
