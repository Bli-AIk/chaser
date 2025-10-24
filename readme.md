# chaser

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/chaser.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/chaser.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> **Status**: 🚧 Initial iteration (features and structure may change frequently)

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

*(Coming soon)*

## Dependencies

This project uses the following crates:

| Crate                                     | Version | Description                                                                                     |
|-------------------------------------------|---------|-------------------------------------------------------------------------------------------------|
| [notify](https://crates.io/crates/notify) | 8.2.0   | Cross-platform filesystem notification library.                                                 |
| [serde](https://crates.io/crates/serde)   | 1.0.228 | A framework for serializing and deserializing Rust data structures efficiently and generically. |

## Contributing

Contributions are welcome! Whether you want to fix a bug, add a feature, or improve the documentation, feel free to:

* Submit an Issue or Pull Request.
* Share ideas and discuss the architecture.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
