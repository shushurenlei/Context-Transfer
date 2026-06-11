<div align="center">
  <h1>🔄 Context Transfer</h1>
  <p><strong>AI 编码助手上下文迁移工具</strong></p>

  <p>
    <img alt="Version" src="https://img.shields.io/badge/version-0.1.6-blue" />
    <img alt="Platform" src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue" />
    <img alt="Built with" src="https://img.shields.io/badge/built%20with-Tauri%202-6366f1" />
    <img alt="Downloads" src="https://img.shields.io/github/downloads/shushurenlei/Context-Transfer/total?color=green" />
    <img alt="License" src="https://img.shields.io/badge/license-MIT-brightgreen" />
  </p>
</div>

---

在 AI 编码助手之间切换时，对话上下文会丢失。Context Transfer 提供原生桌面界面，支持以下工具之间**一键迁移会话上下文**：

<p align="center">
  <strong>Claude Code</strong> &nbsp;↔&nbsp;
  <strong>Codex CLI</strong>
</p>

## 📥 下载

支持 **macOS**、**Windows** 和 **Linux**。

前往 [v0.1.6 Releases](https://github.com/shushurenlei/Context-Transfer/releases/tag/v0.1.6) 页面下载最新版本：

| 文件 | 平台 |
|------|------|
| `Context.Transfer_0.1.6_aarch64.app.zip` | macOS (Apple Silicon，推荐) |
| `Context.Transfer_0.1.6_aarch64.dmg` | macOS (Apple Silicon) |
| `Context.Transfer_0.1.6_x64_en-US.msi` | Windows (x64) |
| `Context.Transfer_0.1.6_amd64.deb` | Linux (Debian/Ubuntu) |

## ✨ 功能特性

| 方向 | 源 → 目标 | 状态 |
|------|-----------|------|
| 正向 | Claude Code → Codex CLI | ✅ |
| 反向 | Codex CLI → Claude Code | ✅ |
| 扩展 | 更多 AI 工具（Claude Desktop, Gemini CLI, OpenCode ...） | 🚧 规划中 |

### 三种注入方式

| 方式 | 原理 | 适用场景 |
|------|------|---------|
| **Prompt 模式** | 生成上下文文本复制到剪贴板 | 临时切换，手动粘贴 |
| **文件写入模式** | 将上下文写入项目配置文件，目标工具启动时自动读取 | 项目内长期使用 |
| **一键自动模式** | 写入配置文件 + 启动目标工具 | 无缝切换 |

### 其他功能

- 🔍 自动扫描已安装 AI 工具的项目历史
- 📋 会话列表按时间排序，一键选择最新
- 👁️ 迁移前预览上下文内容
- 🧹 一键清理迁移写入的内容
- 💾 导出上下文为 Markdown 文件

## 🛠 技术栈

<p>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white" />
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2-FFC131?logo=tauri&logoColor=white" />
  <img alt="React" src="https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black" />
  <img alt="TypeScript" src="https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white" />
  <img alt="Tailwind" src="https://img.shields.io/badge/Tailwind_CSS-38B2AC?logo=tailwind-css&logoColor=white" />
</p>

## 🔧 从源码构建

### 前置要求

- [Rust](https://www.rust-lang.org/tools/install) ≥ 1.77
- [Node.js](https://nodejs.org/) ≥ 20
- **macOS**: Xcode Command Line Tools
- **Windows**: Visual Studio Build Tools（含 C++ 工作负载）
- **Linux**: `libwebkit2gtk-4.1-dev` 等 Tauri 系统依赖

### 构建

```bash
git clone https://github.com/shushurenlei/Context-Transfer.git
cd Context-Transfer

# 安装前端依赖
cd frontend && npm install && cd ..

# 安装 Tauri CLI
cargo install tauri-cli --version "^2"

# 构建
cd frontend && npm run build && cd ..
cd src-tauri && cargo tauri build
```

产物位于 `src-tauri/target/release/bundle/`。

### 开发

```bash
cd frontend && npm run dev     # 终端 1：Vite 开发服务器
cd src-tauri && cargo tauri dev # 终端 2：Tauri 热重载
```

## 📄 License

[MIT](LICENSE)
