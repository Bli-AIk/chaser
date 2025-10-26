# chaser

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/chaser.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/chaser.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

![Chaser](./chaser_logo.svg)

**chaser** is a lightweight file path tracker.

| English         | Chinese                     |
|-----------------|-----------------------------|
| English Version | [简体中文](./readme_zh-hant.md) |

## Introduction

`chaser` is an automated file path synchronization tool. It solves a common pain point: when paths or files referenced in configuration files change, you always need to manually update the corresponding paths in the configuration files.

With `chaser`, you only need to perform simple configuration, and it will automatically monitor changes to specified paths and update related references in configuration files in real-time. It can be closed after use.

In the future, it will also support running as a daemon process.

## Features

* Automatically tracks file and directory path changes (renaming, moving).
* Updates the corresponding paths in specified configuration files.
* Lightweight and easy to configure.
* (Planned) Can be run as a daemon.

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/Bli-AIk/chaser.git
   cd chaser
   ```

3. **Build and run**:
   ```bash
   cargo run
   ```

4. **Basic commands**:
   - Add a path to monitor: `cargo run -- add /path/to/monitor`
   - Remove a path: `cargo run -- remove /path/to/monitor`
   - List monitored paths: `cargo run -- list`
   - Set language: `cargo run -- set-lang en` (or `zh-cn`)
   - Show available languages: `cargo run -- available-lang`

5. **Configuration**:
   The configuration file is automatically created at `~/.config/chaser/config.json` on first run.

## How to Build

### Prerequisites
- Rust 1.70 or later

### Build Steps

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Bli-AIk/chaser.git
   cd chaser
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

4. **Install globally** (optional):
   ```bash
   cargo install --path .
   ```

## Dependencies

This project uses the following crates:

| Crate                                                   | Version | Description                                                   |
|---------------------------------------------------------|---------|---------------------------------------------------------------|
| [notify](https://crates.io/crates/notify)               | 8.2.0   | Real-time file system monitoring for path change detection    |
| [serde](https://crates.io/crates/serde)                 | 1.0.228 | Configuration file serialization and deserialization          |
| [serde_json](https://crates.io/crates/serde_json)       | 1.0.145 | JSON format support for configuration storage                 |
| [serde_yaml_ng](https://crates.io/crates/serde_yaml_ng) | 0.10    | YAML configuration file parsing and writing                   |
| [toml](https://crates.io/crates/toml)                   | 0.8     | TOML format configuration file support                        |
| [csv](https://crates.io/crates/csv)                     | 1.3     | CSV file reading and updating for tabular data                |
| [clap](https://crates.io/crates/clap)                   | 4.0     | Command-line interface with subcommands and options           |
| [dirs](https://crates.io/crates/dirs)                   | 6.0     | Cross-platform system configuration directory discovery       |
| [anyhow](https://crates.io/crates/anyhow)               | 1.0     | Simplified error handling with context and chaining           |
| [sys-locale](https://crates.io/crates/sys-locale)       | 0.3     | System language preference detection for internationalization |
| [owo-colors](https://crates.io/crates/owo-colors)       | 4.0     | Terminal color output for enhanced user experience            |

## Contributing

Contributions are welcome! Whether you want to fix a bug, add a feature, or improve the documentation, feel free to:

* Submit an Issue or Pull Request.
* Share ideas and discuss the architecture.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
