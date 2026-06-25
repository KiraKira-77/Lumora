# Lumora Dock And Launcher Window Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current framed all-in-one window with a real two-layer desktop UI: a bottom Dock window that opens on launch, and a hidden Launcher window that appears from shortcut or the Dock Lumora icon.

**Architecture:** Tauri owns two transparent frameless windows, `dock` and `launcher`. React chooses the rendered surface from the current window label and shares Dock/shortcut data through small local modules. Rust commands toggle the Launcher and route the global shortcut to the Launcher window without making shortcut registration fatal.

**Tech Stack:** Tauri v2, Rust, React 19, TypeScript, Vite, Vitest.

---

## File Structure

- Modify `src-tauri/tauri.conf.json`: define `dock` and `launcher` windows instead of the current `main` window.
- Modify `src-tauri/src/main.rs`: replace `show_launcher_window("main")` behavior with `toggle_launcher_window`, expose a `toggle_launcher` command, and focus the `launcher` window on shortcut.
- Create `src/lib/windowSurface.ts`: detect current Tauri window label with browser-preview fallback.
- Create `src/lib/shortcutSlots.ts`: define keyboard rows and slot helpers.
- Modify `src/App.tsx`: route to `DockSurface` or `LauncherSurface`.
- Create `src/components/DockSurface.tsx`: render only the standalone bottom Dock.
- Create `src/components/LauncherSurface.tsx`: render search plus keyboard shortcut grid.
- Modify `src/App.css`: remove fake desktop frame/background and style only Dock/Launcher transparent surfaces.
- Modify tests under `src/**/*.test.tsx` and `src/lib/*.test.ts`: assert two-window config, Dock-first launch surface, keyboard slot rows, and no large preview frame.

## Task 1: Tauri Window Contract

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src/lib/tauriConfig.test.ts`

- [ ] **Step 1: Write failing config test**

Assert:

- windows include labels `dock` and `launcher`;
- `dock.visible` is not false;
- `launcher.visible` is false;
- both are `transparent`, `decorations: false`, `alwaysOnTop: true`;
- no window label is `main`.

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- --run src/lib/tauriConfig.test.ts`

Expected: FAIL because current config only has `main`.

- [ ] **Step 3: Update Tauri config**

Set:

- `dock`: width about 760, height about 112, bottom-centered intent, transparent, frameless, resizable false, always on top.
- `launcher`: width about 760, height about 520, centered, transparent, frameless, visible false, always on top.

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- --run src/lib/tauriConfig.test.ts`

Expected: PASS.

## Task 2: Shortcut Slot Model

**Files:**
- Create: `src/lib/shortcutSlots.ts`
- Create: `src/lib/shortcutSlots.test.ts`

- [ ] **Step 1: Write failing slot tests**

Assert rows are:

- `1 2 3 4 5 6 7 8 9 0`
- `Q W E R T Y U I O P`
- `A S D F G H J K L`
- `Z X C V B N M`

Assert empty slots expose only key identity and no target.

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- --run src/lib/shortcutSlots.test.ts`

Expected: FAIL because file does not exist.

- [ ] **Step 3: Implement slot model**

Export `keyboardRows`, `shortcutKeys`, and `createEmptyShortcutSlots()`.

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- --run src/lib/shortcutSlots.test.ts`

Expected: PASS.

## Task 3: Window Surface Routing

**Files:**
- Create: `src/lib/windowSurface.ts`
- Create: `src/lib/windowSurface.test.ts`
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`

- [ ] **Step 1: Write failing routing tests**

Assert:

- `App` can render a Dock-only surface.
- `App` can render a Launcher-only surface.
- browser preview defaults to a demo mode that shows both surfaces for development only.
- rendered Dock surface does not contain `desktop-stage`, `Glass Launcher`, or `桌面收纳`.

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- --run src/App.test.tsx src/lib/windowSurface.test.ts`

Expected: FAIL because App currently renders the all-in-one shell.

- [ ] **Step 3: Implement surface routing**

Add `getWindowSurface()` with Tauri label detection where available. Keep a safe browser preview fallback.

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- --run src/App.test.tsx src/lib/windowSurface.test.ts`

Expected: PASS.

## Task 4: Dock Surface

**Files:**
- Create: `src/components/DockSurface.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.css`
- Modify: `src/lib/native.ts`

- [ ] **Step 1: Write failing Dock render test**

Assert Dock renders:

- `aria-label="Lumora Dock"`;
- first button is `aria-label="光枢"`;
- no launcher panel markup;
- no fake desktop background.

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- --run src/App.test.tsx`

Expected: FAIL.

- [ ] **Step 3: Implement Dock surface**

Move Dock rendering into `DockSurface`. Clicking the launcher item calls native command `toggle_launcher`; browser preview may update local demo state.

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- --run src/App.test.tsx`

Expected: PASS.

## Task 5: Launcher Surface

**Files:**
- Create: `src/components/LauncherSurface.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.css`

- [ ] **Step 1: Write failing Launcher render test**

Assert Launcher renders:

- `aria-label="Lumora Launcher"`;
- search input;
- keyboard rows;
- empty slots with key badges;
- no `Folder`, `Widget`, or `App` demo type labels.

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test -- --run src/App.test.tsx src/lib/shortcutSlots.test.ts`

Expected: FAIL until Launcher exists.

- [ ] **Step 3: Implement Launcher surface**

Render a glass panel with top search input and keycap grid. Use Dock items to fill early slots only as a first pass, while preserving empty slots.

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test -- --run src/App.test.tsx src/lib/shortcutSlots.test.ts`

Expected: PASS.

## Task 6: Rust Window Control

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write failing Rust unit test**

Assert the launcher window label helper returns `launcher`, and Dock window label helper returns `dock`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test`

Expected: FAIL until helpers exist.

- [ ] **Step 3: Implement commands**

Add:

- `toggle_launcher`
- `show_launcher_window`
- `hide_launcher_window`

Shortcut handler should show/focus `launcher`, not `main`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test`

Expected: PASS.

## Task 7: Full Verification

**Files:**
- No new files unless verification reveals a defect.

- [ ] **Step 1: Run frontend tests**

Run: `npm test -- --run`

Expected: all tests pass.

- [ ] **Step 2: Run frontend build**

Run: `npm run build`

Expected: TypeScript and Vite build pass.

- [ ] **Step 3: Run Rust checks**

Run: `cargo test` in `src-tauri`

Expected: all tests pass.

Run: `cargo check` in `src-tauri`

Expected: pass.

- [ ] **Step 4: Build Tauri release**

Run: `npm run tauri:build`

Expected: release exe builds.

- [ ] **Step 5: Run release exe**

Run: `src-tauri\target\release\lumora.exe`.

Expected:

- only the Dock window is visible at launch;
- no large framed preview window appears;
- Dock Lumora icon opens Launcher;
- global shortcut opens Launcher when not occupied by another app.
