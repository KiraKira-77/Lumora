# Lumora 开发冻结版产品方案

## 1. 结论

Lumora 第一版不做 Workspace，不做云同步，不做 AI，不做插件市场。

第一版只做一个清晰产品：

> Mac 风格的 Windows 快捷桌面：底部 Dock 插槽 + 玻璃启动器 + 文件搜索 + 桌面整理。

产品目标不是替代 Windows，也不是做效率工具大全，而是解决四个直接问题：

- 桌面图标太乱。
- 常用应用、文件夹、文件、网页打开不够快。
- Windows 搜索不好用。
- 桌面文件缺少快速收纳和分类。

V1 必须先把“拖入、打开、搜索、整理”四件事做顺。后续 Cloud 功能可以做，但不能进入第一版开发范围。

## 2. 产品定位

### 2.1 一句话定位

Lumora 是一个 Mac 风格的 Windows 快捷桌面，用户可以把常用应用、文件、文件夹和网页拖入 Dock 或快捷键面板，通过快捷键、搜索和桌面收纳快速管理自己的电脑桌面。

### 2.2 用户心智

- Dock：放常用入口。
- Launcher：快速打开一切。
- File Search：快速找文件。
- Desktop Organizer：快速清空和整理桌面。
- Cloud Restore：未来换电脑恢复配置。

### 2.3 不要使用的定位

不要主推：

- Workspace 工作状态管理系统。
- AI 效率助手。
- Windows 替代桌面。
- 插件平台。
- 截图工具合集。

这些会让 V1 范围失控。

## 3. 目标用户

### 3.1 核心用户

Windows 桌面重度用户。

典型特征：

- 桌面图标很多。
- 经常从桌面、下载目录、项目目录找文件。
- 经常打开固定应用和固定文件夹。
- 不满意 Windows 开始菜单和搜索。
- 喜欢 Mac Dock、玻璃质感、顺滑动效。
- 愿意配置自己的快捷入口。

### 3.2 首批场景

办公用户：

- 常用微信、钉钉、浏览器、Office、项目文件夹。
- 桌面经常堆临时文件。
- 需要一键收纳桌面。

设计/内容用户：

- 常用设计软件、素材文件夹、截图、下载目录。
- 桌面经常堆图片、PDF、PSD、压缩包。
- 需要快速分类和拖拽入口。

开发/技术用户：

- 常用 IDE、终端、浏览器、项目目录、文档站。
- 需要快速打开项目和文件夹。
- 对快捷键和响应速度敏感。

## 4. V1 产品主线

V1 主线：

> 拖入常用项 → 固定到 Dock 或快捷键矩阵 → 快速打开 → 搜索文件 → 收纳桌面。

首次使用必须围绕这条主线设计：

1. 用户安装 Lumora。
2. Lumora 显示底部 Dock 和启动器。
3. 用户拖入第一个应用或文件夹。
4. Lumora 自动生成 Dock 图标和快捷键绑定建议。
5. 用户按 Alt + Space 呼出启动器。
6. 用户搜索并打开应用/文件/文件夹。
7. Lumora 提示扫描桌面文件。
8. 用户预览分类结果。
9. 用户确认整理，桌面文件进入分类文件夹。
10. 用户可以隐藏 Windows 桌面图标。

## 5. V1 功能范围

### 5.1 Glass Launcher

中央玻璃启动器。

必须做：

- Alt + Space 呼出/隐藏。
- 搜索应用。
- 搜索文件和文件夹。
- 搜索网页快捷项。
- 默认显示快捷键矩阵。
- 支持拖入应用、文件、文件夹、URL。
- 支持按键绑定。
- 支持最近打开。
- 支持从搜索结果固定到 Dock。

暂不做：

- 插件市场。
- AI 对话。
- 复杂命令工作流。
- 云同步。
- 全文内容搜索。

体验要求：

- 呼出动画 150 到 220ms。
- 输入搜索无明显卡顿。
- 搜索结果键盘可操作。
- Esc 关闭。
- Enter 打开第一项。

### 5.2 Bottom Dock 插槽

屏幕中下方的快捷 Dock。

必须做：

- 常驻显示。
- 自动隐藏。
- 拖入应用、文件、文件夹、URL。
- 点击打开。
- 拖拽排序。
- 右键移除。
- 右键编辑名称。
- 右键打开所在位置。
- 从 Launcher 固定项目到 Dock。

暂不做：

- 多 Dock。
- 复杂分组。
- Dock 插件。
- 跨屏幕复杂规则。
- 像 macOS 一样完整模拟放大效果。

体验要求：

- 不遮挡任务栏时可用。
- 自动隐藏要稳定。
- 拖拽反馈必须明确。
- 图标加载失败时有默认图标。

### 5.3 File Search

文件搜索是核心能力，但不自研索引。

推荐方案：

- V1 优先集成 Everything。
- 如果本机未安装 Everything，提示用户安装或使用降级搜索。
- Lumora 负责 UI、动作和固定入口。

必须做：

- 搜文件名。
- 搜文件夹名。
- 打开文件。
- 打开所在目录。
- 复制路径。
- 固定到 Dock。
- 按类型过滤：应用、文件夹、文档、图片、压缩包。

暂不做：

- 全文搜索。
- 云盘搜索。
- 复杂查询语法。
- 自建全盘索引。

### 5.4 Desktop Organizer

桌面整理是差异化功能，也是风险最高的功能。

必须做：

- 扫描桌面文件。
- 分类预览。
- 用户确认后移动。
- 整理后可撤销。
- 保留整理日志。
- 支持 Inbox 收纳箱。

默认分类：

- Images
- Docs
- Archives
- Installers
- Videos
- Projects
- Inbox

文件动作原则：

- 不静默移动文件。
- 不删除用户文件。
- 不覆盖同名文件。
- 移动前必须展示目标路径。
- 移动后必须可撤销。
- 失败项必须明确展示。

### 5.5 Settings

必须做：

- 启动器快捷键。
- Dock 显示模式：常驻、自动隐藏、仅桌面显示。
- 玻璃强度。
- 动画开关。
- 桌面整理目标目录。
- Everything 集成状态。
- 配置导入导出。

暂不做：

- 账号系统。
- 云同步。
- 订阅管理。
- 复杂主题市场。

## 6. V1 明确不做

V1 不做：

- Workspace。
- Cloud Sync。
- 新电脑一键复现。
- AI 助手。
- 插件市场。
- 复杂截图编辑。
- 完整剪贴板管理。
- 团队功能。
- 云盘文件搜索。
- 应用商店。
- 自动安装缺失应用。

这些功能不进入第一版，避免开发范围失控。

## 7. V2 Cloud 方向

Cloud 是后续强付费点，但必须建立在 V1 本地体验成立之后。

### 7.1 Cloud 可同步内容

可以同步：

- Dock 配置。
- Launcher 快捷键矩阵。
- 主题和玻璃强度。
- 桌面整理规则。
- 文件分类规则。
- 小组件布局。
- 常用网页。
- 应用识别信息。
- 自定义路径变量。

不能直接同步：

- 本机文件本体。
- 软件登录状态。
- 软件内部配置。
- 浏览器登录态。
- 付费软件授权。

### 7.2 路径抽象

不能只保存绝对路径。

配置中应优先保存：

- `%USERPROFILE%`
- `%DESKTOP%`
- `%DOWNLOADS%`
- `%DOCUMENTS%`
- `%PICTURES%`
- 自定义变量：`PROJECTS_DIR`

目的：

- 换电脑后减少路径失效。
- 允许用户重新绑定目录。
- 避免云配置只能在一台电脑上有效。

### 7.3 应用恢复

换电脑后自动恢复应用只能尽力而为，不能承诺 100%。

可做：

- 扫描本机已安装应用。
- 对比云端配置。
- 识别缺失应用。
- 通过 winget 安装可识别应用。
- 显示需要手动安装的应用。
- 安装完成后恢复 Dock 和快捷键。

不能承诺：

- 所有软件都能自动安装。
- 所有软件都能静默安装。
- 付费软件能自动授权。
- 软件数据和插件能完整恢复。

恢复报告必须分组：

- 已可用。
- 可自动安装。
- 需要手动安装。
- 路径需要重新绑定。
- 恢复失败。

### 7.4 Cloud 阶段划分

V1.5：

- 本地配置导出。
- 本地配置导入。
- 缺失应用检测。
- winget 安装建议。

V2：

- 账号系统。
- 配置上云。
- 多设备配置同步。
- 配置版本历史。
- 本地加密后上传。

V2.5：

- 新电脑恢复向导。
- winget 自动安装。
- 路径重绑定。
- 恢复报告。

## 8. 技术路线

主仓库应同时放前端和本地后端。

推荐技术栈：

- Tauri v2
- React
- TypeScript
- Vite
- Rust

仓库性质：

> Lumora 桌面客户端仓库，不是纯前端仓库。

建议结构：

```text
Lumora/
  package.json
  vite.config.ts
  tsconfig.json
  src/
    components/
      Launcher/
      Dock/
      DesktopOrganizer/
      FileSearch/
      Settings/
    lib/
    styles/
  src-tauri/
    Cargo.toml
    tauri.conf.json
    src/
      main.rs
      commands/
        app_launcher.rs
        config_store.rs
        desktop_organizer.rs
        file_search.rs
        shortcuts.rs
  docs/
    product/
    design/
    plans/
```

职责划分：

React 前端：

- 玻璃启动器 UI。
- Dock UI。
- 快捷键矩阵。
- 文件搜索结果。
- 桌面整理预览。
- 设置页。

Rust/Tauri 本地后端：

- 打开应用、文件、文件夹、URL。
- 扫描桌面。
- 移动文件。
- 撤销整理。
- 读取和保存本地配置。
- 调用 Everything。
- 注册全局快捷键。
- 系统托盘。
- 开机启动。

## 9. 数据模型草案

### 9.1 DockItem

```json
{
  "id": "dock_chrome",
  "type": "app",
  "name": "Chrome",
  "icon": "auto",
  "target": "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
  "args": [],
  "hotkey": "C",
  "order": 1,
  "createdAt": "2026-06-25T00:00:00Z"
}
```

### 9.2 LauncherBinding

```json
{
  "key": "W",
  "actionType": "open",
  "targetId": "dock_wechat",
  "label": "微信"
}
```

### 9.3 OrganizerRule

```json
{
  "id": "rule_images",
  "name": "图片",
  "extensions": [".png", ".jpg", ".jpeg", ".webp", ".gif"],
  "targetDir": "%DESKTOP%\\Lumora\\Images"
}
```

### 9.4 AppIdentity

```json
{
  "name": "Google Chrome",
  "localPath": "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
  "wingetId": "Google.Chrome",
  "msStoreId": null,
  "officialUrl": "https://www.google.com/chrome/",
  "detectRules": [
    "process:chrome.exe",
    "path:%PROGRAMFILES%\\Google\\Chrome\\Application\\chrome.exe"
  ]
}
```

## 10. MVP 开发分期

### Phase 0：项目骨架

目标：

- 建立 Tauri + React + TypeScript 项目。
- 能运行桌面窗口。
- 能展示基础玻璃 UI。

验收：

- `npm run tauri dev` 能启动。
- 主窗口显示 Lumora 启动器静态 UI。

### Phase 1：本地配置和 Dock

目标：

- 实现 Dock 插槽。
- 支持添加、删除、排序。
- 保存到本地配置。

验收：

- 重启应用后 Dock 配置不丢失。
- 点击 Dock 项能打开目标。

### Phase 2：Launcher

目标：

- 实现 Alt + Space 呼出。
- 实现快捷键矩阵。
- 实现应用/配置项搜索。

验收：

- 快捷键能呼出。
- Enter 能打开第一项。
- Esc 能关闭。

### Phase 3：File Search

目标：

- 集成 Everything 或提供可替换搜索接口。
- 搜索结果可打开、复制路径、固定到 Dock。

验收：

- 搜索文件名能返回结果。
- 文件动作可执行。

### Phase 4：Desktop Organizer

目标：

- 扫描桌面。
- 生成分类预览。
- 执行整理。
- 支持撤销。

验收：

- 整理前有预览。
- 整理后文件进入目标目录。
- 撤销后文件回到原位置。
- 失败项明确展示。

### Phase 5：打磨和发布准备

目标：

- 设置页。
- 图标加载。
- 动效优化。
- 打包安装。

验收：

- 能打包 Windows 安装包。
- 核心流程无明显卡顿。

## 11. 成功指标

V1 核心指标：

- 用户是否隐藏或清空 Windows 桌面图标。
- 用户每天通过 Lumora 打开应用/文件的次数。
- 用户是否持续使用 Dock。
- 用户是否使用桌面整理。

具体指标：

- D1 留存。
- D7 留存。
- 每日 Launcher 呼出次数。
- 每日 Dock 点击次数。
- 用户平均固定项数量。
- 桌面整理使用次数。
- 桌面整理撤销率。
- 搜索成功打开率。

## 12. 风险和约束

### 12.1 不要变成皮肤工具

Mac 风格只是入口吸引力，不是产品壁垒。

真正壁垒必须是：

- 打开快。
- 搜索快。
- 整理稳。
- 配置不丢。

### 12.2 不要静默移动文件

桌面整理必须保守。

任何移动文件动作都必须：

- 预览。
- 确认。
- 记录日志。
- 可撤销。

### 12.3 不要一开始做云

云同步会引入账号、加密、冲突合并、隐私、服务端成本。

V1 应先做本地导入导出，为 Cloud 预留数据结构。

### 12.4 不要自研文件索引

Everything 已经解决文件名搜索性能问题。

Lumora 应做体验整合，不应在 V1 重造索引系统。

## 13. 开发前最终边界

V1 必须做：

- Tauri + React 桌面应用。
- 玻璃启动器。
- 底部 Dock。
- 拖入应用/文件/文件夹/URL。
- 本地配置保存。
- 快捷键矩阵。
- 文件搜索。
- 桌面整理预览。
- 一键分类。
- 撤销整理。
- 基础设置。

V1 不做：

- 云同步。
- 账号登录。
- 自动安装缺失应用。
- Workspace。
- AI。
- 插件市场。
- 复杂截图。
- 完整剪贴板管理。

如果开发过程中出现范围争议，以这一条为准：

> V1 只验证“Windows 桌面能否被 Lumora 变得更干净、更快、更顺手”。
