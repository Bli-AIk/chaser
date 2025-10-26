# chaser

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/chaser.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/chaser.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

![Chaser](./chaser_logo.svg)

**chaser** 是一个轻量的文件路径追踪器。

| 英语                     | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 简介

`chaser` 是一个自动化的文件路径同步工具。它解决了一个常见的痛点：当配置文件中引用的路径或文件发生变化时，总是需要手动更新配置文件中的对应路径。

通过 `chaser`，你只需进行简单配置，它就能自动监控指定路径的变化，并实时更新配置文件中的相关引用。使用完毕后即可关闭。

在未来，它也将支持并允许作为守护进程持续运行。

## 功能

* 自动追踪文件和目录的路径变动（重命名、移动）。
* 自动更新指定配置文件中的对应路径。
* 轻量、易于配置。
* （计划中）支持作为守护进程运行。

## 如何使用

1. **安装 Rust**（如果尚未安装）：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **克隆仓库**：
   ```bash
   git clone https://github.com/Bli-AIk/chaser.git
   cd chaser
   ```

3. **构建并运行**：
   ```bash
   cargo run
   ```

4. **基本命令**：
    - 添加监控路径：`cargo run -- add /path/to/monitor`
    - 移除路径：`cargo run -- remove /path/to/monitor`
    - 列出监控路径：`cargo run -- list`
    - 设置语言：`cargo run -- set-lang zh-cn`（或 `en`）
    - 显示可用语言：`cargo run -- available-lang`

5. **配置**：
   配置文件会在首次运行时自动创建在 `~/.config/chaser/config.json`

## 如何构建

### 前置要求

- Rust 1.70 或更高版本

### 构建步骤

1. **克隆仓库**：
   ```bash
   git clone https://github.com/Bli-AIk/chaser.git
   cd chaser
   ```

2. **构建项目**：
   ```bash
   cargo build --release
   ```

3. **运行测试**：
   ```bash
   cargo test
   ```

4. **全局安装**（可选）：
   ```bash
   cargo install --path .
   ```

## 依赖项

本项目使用了以下 crate：

| Crate                                                   | 版本      | 描述                  |
|---------------------------------------------------------|---------|---------------------|
| [notify](https://crates.io/crates/notify)               | 8.2.0   | 实时监控文件系统变化，检测路径修改   |
| [serde](https://crates.io/crates/serde)                 | 1.0.228 | 处理配置文件的序列化和反序列化     |
| [serde_json](https://crates.io/crates/serde_json)       | 1.0.145 | 为配置存储提供 JSON 格式支持   |
| [serde_yaml_ng](https://crates.io/crates/serde_yaml_ng) | 0.10    | 支持 YAML 配置文件的解析和写入  |
| [toml](https://crates.io/crates/toml)                   | 0.8     | 支持 TOML 格式的配置文件     |
| [csv](https://crates.io/crates/csv)                     | 1.3     | 处理表格数据的 CSV 文件读取和更新 |
| [clap](https://crates.io/crates/clap)                   | 4.0     | 提供带子命令和选项的命令行界面     |
| [dirs](https://crates.io/crates/dirs)                   | 6.0     | 跨平台定位系统配置目录         |
| [anyhow](https://crates.io/crates/anyhow)               | 1.0     | 简化错误处理，提供上下文和链式功能   |
| [sys-locale](https://crates.io/crates/sys-locale)       | 0.3     | 检测系统语言偏好，支持国际化      |
| [owo-colors](https://crates.io/crates/owo-colors)       | 4.0     | 为终端输出添加色彩，提升用户体验    |

## 贡献

欢迎任何形式的贡献！无论你是想修复 Bug、添加功能还是改进文档，都欢迎：

* 提交 Issue 或 Pull Request！
* 分享想法、讨论架构！

## 许可证

本项目根据您的选择，在以下任一许可下授权：

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
