# Dock 与微信激活问题排障记录

日期：2026-06-26

## 背景

这轮问题集中在 Lumora Dock 的行为：

- 外部应用拖入 Dock 后，图标、顺序、桌面快捷方式隐藏/还原需要稳定。
- Dock 图标点击应像 Windows 任务栏或 macOS Dock 一样：应用已运行时激活现有应用，而不是重复启动。
- 微信收到消息时，Dock 图标需要有提醒状态；点击查看后提醒状态要清除。
- Dock 和启动器当前视觉样式不应被大改，只能在原有基础上改行为。

## 遇到的问题

### 1. 拖入 Dock 后位置不对

现象：从桌面拖应用到 Dock 时，用户希望放到哪个位置就插入哪个位置，但旧逻辑默认追加到右侧。

处理：

- 前端增加 Dock 指针位置到插入下标的计算。
- 拖入外部文件、应用快捷方式和 Dock 内部重排都复用插入位置。
- 固定图标仍保持边界：启动器固定在左侧，废纸篓固定在右侧。

相关文件：

- `src/lib/dockDropPosition.ts`
- `src/lib/dockItems.ts`
- `src/components/DockSurface.tsx`
- `src/App.tsx`

### 2. 拖入应用后图标丢失

现象：腾讯视频、腾讯 QQ、微信等部分应用拖入 Dock 后出现无图标。

处理：

- 后端描述 drop target 时提取 Windows 应用/快捷方式图标并缓存为 PNG。
- 前端渲染 Dock 图标时优先使用 `iconPath`，没有图标时才使用默认字母/类型图标。

相关文件：

- `src-tauri/src/main.rs`
- `src/lib/native.ts`
- `src/components/DockSurface.tsx`

### 3. 桌面快捷方式隐藏/还原逻辑不稳定

现象：应用拖入 Dock 后，桌面快捷方式有时没有被移走；从 Dock 移除后也需要还原到桌面。

处理：

- 拖入 Dock 时，将桌面来源文件移动到桌面下的 `.lumora_dock_hidden`。
- Dock item 记录 `originalDesktopPath`。
- 从 Dock 移除时，将隐藏文件移动回原桌面路径。
- 同时兼容用户桌面和公共桌面路径。

相关文件：

- `src-tauri/src/main.rs`
- `src/lib/dockItems.ts`
- `src/App.tsx`

### 4. 微信已登录时，Dock 点击却弹登录/进入微信窗口

现象：微信已经登录并在托盘运行，点击 Dock 微信后弹出“进入微信”窗口，等同于重新启动微信，而不是恢复当前聊天主窗口。

根因：

- Lumora 原逻辑是按 `Weixin.exe` 枚举 HWND，然后 `ShowWindow` / `SetForegroundWindow`。
- 微信 Windows 客户端是 Qt 多窗口应用，同一进程下存在大量内部窗口：
  - `Qt51514WxTrayIconMessageWindowClass`
  - `Qt51514QWindowToolSaveBits`
  - `Chrome_SystemMessageWindow`
  - IME / 搜狗输入法窗口
  - 图片/视频容器
  - 隐藏或离屏的 `Qt51514QWindowIcon`
- Windows 右下角托盘点击不是这样枚举窗口，而是触发微信自己的托盘回调。因此托盘能恢复，Dock 枚举 HWND 会选错。

### 5. 微信空白大窗口/灰色窗口

现象：某次修复尝试后，点击 Dock 微信弹出灰色空白大窗口。

错误路线：

- 曾尝试把“隐藏的大尺寸微信 Qt 窗口”当成主窗口强制恢复。
- 结果证明这条路不稳定，会把微信内部隐藏容器拉出来，形成灰色空壳。

结论：

- 不能继续靠 class、标题、尺寸猜微信 HWND。
- 这条路线已经废弃，代码中明确禁止隐藏微信窗口作为激活候选。

### 6. 微信消息提醒状态

需求：消息来时 Dock 图标跳动两次并显示橙色提醒底色/提示；点击查看后提醒消失。

处理：

- Windows 后端注册 shell hook。
- 捕获窗口 flash 类事件后，将对应应用标记为 attention。
- Dock 前端定时刷新运行状态，也监听后端 attention 变化事件。
- 点击 Dock 图标时清除该 target 的 attention 状态。

相关文件：

- `src-tauri/src/main.rs`
- `src/App.tsx`
- `src/components/DockSurface.tsx`
- `src/App.css`

## 最终微信方案

最终方案不再把微信当普通窗口应用处理。

### 1. 普通应用

普通应用仍走：

1. 根据 Dock target 解析启动身份。
2. 枚举安全的顶层窗口。
3. 激活已有窗口。
4. 找不到窗口时才启动 target。

### 2. 微信这类托盘托管应用

对 `weixin.exe` / `wechat.exe`：

1. 先尝试通过 Windows 托盘 toolbar 找微信托盘按钮。
2. 读取 Explorer 托盘 `ToolbarWindow32` 的按钮文本。
3. 匹配个人微信，排除企业微信。
4. 命中后向托盘按钮发送点击消息，让微信走自己的托盘恢复逻辑。
5. 如果微信进程已经在跑，但找不到安全托盘按钮，则不再启动 `Weixin.exe`，避免继续弹登录窗口或灰色空壳。

关键点：

- 不再强拉隐藏微信 HWND。
- 不再用 `Weixin.exe` 二次启动来“兜底”已运行微信。
- 微信恢复主界面要尽量走托盘逻辑，而不是窗口猜测逻辑。

相关代码：

- `activate_or_open_target`
- `try_activate_tray_icon_for_identity`
- `tray_toolbar_buttons`
- `read_tray_toolbar_buttons`
- `should_suppress_open_after_activation_miss`
- `hidden_large_wechat_windows_are_not_activation_candidates`

## 验证结果

已执行：

```powershell
cargo test --manifest-path src-tauri\Cargo.toml
npm test -- src/lib/dockItems.test.ts src/lib/dockDropPosition.test.ts src/components/DockSurface.test.tsx
cargo check --manifest-path src-tauri\Cargo.toml
npm run build
cargo build --manifest-path src-tauri\Cargo.toml
git diff --check
```

结果：

- Rust 后端测试：28 passed
- Dock 前端相关测试：15 passed
- Rust 编译检查通过
- 前端构建通过
- Debug 版 `lumora.exe` 构建通过
- `git diff --check` 通过；只出现 Git 的 CRLF 提示

## 后续维护原则

- 不要再通过“隐藏微信窗口 + class/尺寸猜测”恢复微信。
- 不要在微信已运行时用 `open_target` 启动 `Weixin.exe` 作为兜底。
- 新增托盘型应用支持时，应先判断是否能走托盘/应用自身单实例唤醒机制。
- Dock 样式暂时保持现状，后续只在现有基础上微调。
