# RustMC Launcher

![Status](https://img.shields.io/badge/Status-Alpha-orange)
![Platform](https://img.shields.io/badge/Platform-Windows_11-blue)
![Tech Stack](https://img.shields.io/badge/Tech-Rust_|_Tauri_|_React_|_Fluent_UI-000000?logo=rust)

**版本**：v0.1.0（MVP 阶段开发中）  
**目标平台**：Windows 11（优先原生 Fluent 风格，后续扩展 Linux/macOS）  
**核心定位**：**纯启动器**（非自定义客户端），支持**正版 Microsoft 登录 + 离线/盗版模式**（后者为主要用户群体）。高性能、低内存、Win11 原生外观，集成模组/整合包市场、性能优化预设、实例管理等全能功能。参考 Prism Launcher（功能最全基准） + Nitrolaunch（Rust 现代架构）构建。

---

## 🚀 快速开始 (本地开发)

**环境要求**：
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (Cargo)
- [Tauri 依赖](https://tauri.app/v1/guides/getting-started/prerequisites) (在 Windows 上通常只需 C++ Build Tools 和 WebView2)

```bash
# 1. 克隆仓库
git clone https://github.com/WEP-56/RMCL.git
cd RMCL/rustmc-launcher

# 2. 安装前端依赖
npm install

# 3. 启动开发服务器 (自动编译 Rust 后端)
npm run tauri dev
```

---

## 1. 项目概述
这是一个**从零基于 Rust crate 开发的 Minecraft 第三方启动器**。  
它不修改 MC 核心 jar，只负责下载、配置、启动游戏（符合 Mojang 第三方启动器政策）。  

**核心优势**：
- 支持**正版 + 离线/盗版**双模式（离线模式直接用用户名生成 UUID v3）。
- GUI 追求 Win11 原生（Mica 毛玻璃、圆角、Fluent 控件、系统标题栏）。
- 极致性能：Rust + 轻量前端，基于 `reqwest` + `tokio` 的高并发资源下载引擎。
- 完全开源（GPL-3.0 或 MIT），便于社区贡献。

**参考项目**（功能全部来自这些）：
- **Prism Launcher**（MultiMC 分支）：实例管理、模组市场、Modpack 支持的最强基准。
- **Nitrolaunch**：Rust + 插件系统、现代包管理。

---

## 2. 技术栈（Rust 优先，高性能低内存）
| 模块          | 技术栈 | 理由 |
|---------------|--------|------|
| **核心后端** | Rust（`tokio` + `reqwest` + `serde`） | 异步、高性能、内存安全，实现高并发资源下载。 |
| **MC 启动逻辑** | 自实现 Mojang 协议解析 | 纯手写 Manifest Resolver，解析 JSON，提取 Natives，拼接 JVM 参数，控制粒度更强。 |
| **GUI**      | **Tauri 2.0** + React + Fluent UI v9 | 轻量 WebView，完美仿 Win11 Mica/Acrylic + 圆角；内存远低于 Electron。 |
| **API 调用** | `reqwest` + Modrinth API v2 | 实现了搜索、获取版本、一键下载 mod。 |
| **Java 管理** | Adoptium (Temurin) API v3 | 自动探测 OS 架构并获取最新 JRE 下载直链。 |
| **存储**     | JSON (`serde_json`) | 实例、账号本地轻量持久化。 |

---

## 3. 开发路线图 & 进度

- [x] **阶段 1（MVP）**：基础启动 + 离线登录 + 简单实例创建/启动。
  - 实现了基于 UUID v3 的离线登录。
  - 实现了实例数据隔离。
  - 实现了纯 Rust 的并发下载器 (Libraries, Assets)。
  - 实现了 Zip 压缩包的 Natives 自动提取。
  - 实现了 JVM 和 Game 参数占位符替换引擎。
  - 实现了 `tokio::process::Command` 启动 Java，并接管日志通过 Tauri 发送至前端。
- [x] **阶段 4 (部分)**：Win11 UI 打磨。
  - 接入 `@fluentui/react-components`，完成了深色模式、侧边栏导航。
  - 完成了高颜值的实例列表、账号管理面板。
- [x] **阶段 2**：Modrinth 市场 + Mod 安装 + Java 管理。
  - 对接 Modrinth API，实现前端瀑布流模组搜索与详情查看。
  - 实现了一键安装 Mod 至指定实例的 `mods` 文件夹。
  - 对接 Adoptium API 实现了 Java 下载直链获取。
- [x] **阶段 3**：完整实例管理 + Modpack + 性能预设。
  - 接入 Fabric Meta API，实现 Fabric Loader 自动安装。
  - 实现性能预设：创建 Fabric 实例时可选一键自动下载安装 Sodium, Iris, Lithium 优化模组。
- [ ] **阶段 5**：CLI、迁移、自动更新、上架 GitHub。

---

## 4. 系统架构
```text
rustmc-launcher/
├── src/               # React 前端代码 (Fluent UI)
│   ├── components/    # 可复用组件 (如 Sidebar)
│   └── pages/         # 路由页面 (Home, Instances, Accounts, Market)
└── src-tauri/         # Rust 后端代码 (Tauri Core)
    ├── src/
    │   ├── core/      # 核心逻辑 (下载、解析、启动、Modrinth API)
    │   └── models/    # 数据模型 (Account, Instance, Manifest, Modrinth)
    └── tauri.conf.json
```

## 5. 注意事项
- **合法性**：离线模式仅限 cracked 服务器/单机。不要分发修改后 MC jar。
- **性能目标**：启动器本体 + 启动过程内存 < 200MB。
