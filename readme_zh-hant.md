# chaser

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/chaser.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/chaser.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> **状态**：🚧 初始开发阶段（项目结构仍在快速演进中）

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

*（待补充）*

## 依赖项

本项目使用了以下 crate：

| Crate                                     | 版本      | 描述                               |
|-------------------------------------------|---------|----------------------------------|
| [notify](https://crates.io/crates/notify) | 8.2.0   | 跨平台文件系统监控库。                      |
| [serde](https://crates.io/crates/serde)   | 1.0.228 | 一个用于高效、通用地序列化和反序列化 Rust 数据结构的框架。 |

## 贡献

欢迎任何形式的贡献！无论你是想修复 Bug、添加功能还是改进文档，都欢迎：

* 提交 Issue 或 Pull Request！
* 分享想法、讨论架构！

## 许可证

本项目根据您的选择，在以下任一许可下授权：

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
