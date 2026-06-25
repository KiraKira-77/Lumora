# 光枢 Lumora

Lumora 是一个面向 Windows 的桌面效率层原型，目标是把常用应用启动、文件搜索和桌面整理收束到一个更接近 macOS Dock + Launcher 的体验里。

当前仓库是 Tauri + React 实现的桌面客户端。

## 当前状态

已实现：

- 底部无边框 Dock 窗口，默认只显示光枢入口和固定垃圾桶。
- 点击 Dock 左侧光枢图标可唤起 Launcher。
- Launcher 是独立无边框窗口，支持手动关闭和拖动。
- Launcher 内置键盘式快捷槽布局：`1-0`、`Q-P`、`A-L`、`Z-M`。
- 快捷槽默认空置，只显示键位角标。
- 支持拖入文件、文件夹、应用路径到 Dock。
- 支持基础文件搜索和桌面文件分类整理能力。
- 支持全局快捷键唤起 Launcher，当前为 `Alt+Space`。如果系统或其他软件占用，该注册失败不会导致应用崩溃。

尚未实现：

- 从 Windows 应用中提取真实图标。
- 将应用直接拖到某个快捷键槽并持久绑定。
- 云同步配置。
- 换机后一键复现应用安装状态。
- 完整设置页、开机自启、安装包发布流程。

## 技术栈

- Tauri 2
- React 19
- TypeScript
- Vite
- Rust
- Vitest

## 开发环境

需要先安装：

- Node.js
- Rust
- Tauri 2 所需的 Windows 构建依赖

安装前端依赖：

```bash
npm install
```

启动 Web 预览：

```bash
npm run dev
```

启动 Tauri 开发模式：

```bash
npm run tauri:dev
```

## 常用命令

运行测试：

```bash
npm test -- --run
```

前端构建：

```bash
npm run build
```

Rust 测试：

```bash
cd src-tauri
cargo test
```

打包桌面程序：

```bash
npm run tauri:build
```

打包产物位置：

```text
src-tauri/target/release/lumora.exe
```

## 项目结构

```text
src/
  components/          React UI 组件
  lib/                 Dock、快捷槽、存储、原生调用等逻辑
  assets/              Lumora 图标资源
src-tauri/
  src/main.rs          Tauri 原生能力、窗口控制、桌面扫描、文件搜索
  tauri.conf.json      Tauri 窗口与构建配置
docs/
  product/             产品方案
  superpowers/         设计与开发过程文档
```

## 窗口模型

Lumora 当前有两个 Tauri 窗口：

- `dock`：启动后显示在屏幕底部，作为常驻入口。
- `launcher`：默认隐藏，通过 Dock 图标或快捷键唤起。

两个窗口都配置为无边框、透明、置顶、跳过任务栏。Dock 会在原生层先定位到底部再显示，避免启动时出现在屏幕中间。

## 产品方向

短期目标不是复制一个普通启动器，而是做一个 Windows 桌面层：

- Dock 管常用入口。
- Launcher 管搜索和快捷键启动。
- 桌面整理管文件归档。
- 后续配置上云，用于换机复现桌面工作流。

## 已知限制

- Windows WebView2 的透明合成在不同系统设置下可能仍出现边缘异常，需要继续做原生窗口样式验证。
- 真实应用图标提取需要接 Windows Shell API，目前 Dock 中用户拖入项仍使用文字占位。
- `Alt+Space` 在很多机器上会被系统或其他工具占用，需要后续提供快捷键设置入口。

