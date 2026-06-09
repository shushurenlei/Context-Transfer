# Context Reset

🔄 Claude Code → Codex CLI 上下文迁移工具（Tauri 桌面应用）

在 [Claude Code](https://claude.ai/code) 和 [Codex CLI](https://github.com/anthropics/codex) 之间切换时，对话上下文会丢失。Context Reset 提供原生桌面界面，一键提取 Claude Code 的会话上下文并注入到 Codex。

## 功能特性

- 🔍 **自动检测** Claude Code 项目
- 📋 **会话列表** 浏览所有历史会话，按时间排序
- 👁️ **上下文预览** 迁移前预览对话内容
- 🚀 **一键迁移** 支持三种注入方式：
  - **Prompt 模式** — 生成 Prompt 文本，复制到剪贴板
  - **AGENTS.md 模式** — 写入项目 AGENTS.md，Codex 自动读取
  - **自动模式** — 写入 AGENTS.md 并启动 Codex
- 🧹 **清理** 一键清除 AGENTS.md 中的迁移内容
- 💾 **导出** 将上下文导出为 Markdown 文件

## 技术栈

| 层 | 技术 |
|---|------|
| 桌面框架 | [Tauri 2](https://v2.tauri.app/) |
| 后端 | Rust |
| 前端 | React 19 + TypeScript |
| 样式 | Tailwind CSS |
| 构建 | Vite |

## 从源码构建

### 前置要求

- [Rust](https://www.rust-lang.org/tools/install) (≥ 1.77)
- [Node.js](https://nodejs.org/) (≥ 20)
- macOS 需安装 Xcode Command Line Tools

### 构建步骤

```bash
# 1. 克隆仓库
git clone https://github.com/your-username/context-reset.git
cd context-reset

# 2. 安装前端依赖
cd frontend
npm install
cd ..

# 3. 安装 Tauri CLI
cargo install tauri-cli --version "^2"

# 4. 构建前端
cd frontend && npm run build && cd ..

# 5. 构建桌面应用
cd src-tauri
cargo tauri build
```

构建产物：

- `src-tauri/target/release/bundle/macos/Context Reset.app` — macOS 应用
- `src-tauri/target/release/bundle/dmg/Context Reset_*.dmg` — DMG 安装包

### 开发模式

```bash
# 启动开发服务器（前端热更新）
cd frontend && npm run dev

# 在另一个终端启动 Tauri 开发模式
cd src-tauri && cargo tauri dev
```

## 项目结构

```
context_reset/
├── frontend/          # React + TypeScript 前端
│   ├── src/
│   │   ├── api/       # Tauri invoke 封装
│   │   ├── components/
│   │   │   ├── Header.tsx
│   │   │   ├── SessionList.tsx
│   │   │   ├── ContextPreview.tsx
│   │   │   └── MigratePanel.tsx
│   │   ├── App.tsx
│   │   └── main.tsx
│   └── package.json
├── src-tauri/         # Rust + Tauri 后端
│   ├── src/
│   │   ├── main.rs              # 入口
│   │   ├── lib.rs               # Tauri Builder 配置
│   │   ├── commands.rs          # Tauri 命令
│   │   ├── claud_extractor.rs   # Claude Code 会话解析
│   │   ├── context_formatter.rs # 上下文格式化
│   │   └── codex_injector.rs   # Codex 注入器
│   ├── Cargo.toml
│   └── tauri.conf.json
├── AGENTS.md
└── README.md
```

## License

MIT
