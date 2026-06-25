# Lumora Tauri MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first runnable Tauri + React desktop client skeleton for Lumora.

**Architecture:** The app is a single Tauri desktop client. React owns the Launcher, Dock, Desktop Organizer preview, and Settings UI. Rust/Tauri owns native commands later; the first phase exposes only a minimal app-info command and keeps file-moving behavior out of scope.

**Tech Stack:** Tauri v2, React, TypeScript, Vite, Rust, Vitest.

---

## File Structure

- Create `package.json`: scripts and JS dependencies.
- Create `index.html`: Vite entry.
- Create `vite.config.ts`: React and Vitest config.
- Create `tsconfig.json`, `tsconfig.node.json`: TypeScript config.
- Create `src/main.tsx`: React bootstrap.
- Create `src/App.tsx`: first Lumora shell UI.
- Create `src/App.css`: glass launcher and dock styling.
- Create `src/lib/desktopOrganizer.ts`: pure file classification rules.
- Create `src/lib/desktopOrganizer.test.ts`: tests for classification.
- Create `src-tauri/Cargo.toml`: Rust/Tauri dependencies.
- Create `src-tauri/build.rs`: Tauri build hook.
- Create `src-tauri/tauri.conf.json`: app window config.
- Create `src-tauri/src/main.rs`: minimal Tauri command setup.

## Task 1: Scaffold Project Files

**Files:**
- Create: `package.json`
- Create: `index.html`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `tsconfig.node.json`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/App.css`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/build.rs`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`

- [ ] **Step 1: Create minimal project skeleton**
- [ ] **Step 2: Install dependencies with `npm install`**
- [ ] **Step 3: Run `npm run build`**
- [ ] **Step 4: Run `cargo check` inside `src-tauri`**

## Task 2: Add Tested Desktop Classification Rules

**Files:**
- Create: `src/lib/desktopOrganizer.test.ts`
- Create: `src/lib/desktopOrganizer.ts`

- [ ] **Step 1: Write failing tests**

Run: `npm test -- src/lib/desktopOrganizer.test.ts`

Expected: FAIL because `desktopOrganizer.ts` does not exist.

- [ ] **Step 2: Implement minimal classifier**

Rules:

- Images: `.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, `.svg`
- Docs: `.pdf`, `.doc`, `.docx`, `.xls`, `.xlsx`, `.ppt`, `.pptx`, `.txt`, `.md`
- Archives: `.zip`, `.rar`, `.7z`, `.tar`, `.gz`
- Installers: `.exe`, `.msi`
- Videos: `.mp4`, `.mov`, `.mkv`, `.avi`
- Projects: `.sln`, `.csproj`, `.package`, `.json`, `.ts`, `.tsx`, `.rs`
- Inbox: unknown or extensionless files

- [ ] **Step 3: Run tests and confirm pass**

Run: `npm test -- src/lib/desktopOrganizer.test.ts`

Expected: PASS.

## Task 3: First UI Shell

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.css`

- [ ] **Step 1: Render Lumora shell**

Include:

- Mac-inspired glass background.
- Central Glass Launcher.
- Keyboard shortcut matrix.
- Bottom Dock.
- Desktop Organizer preview panel.

- [ ] **Step 2: Run build**

Run: `npm run build`

Expected: TypeScript and Vite build pass.

## Task 4: Tauri Verification

**Files:**
- Modify only if verification reveals config errors.

- [ ] **Step 1: Run Rust check**

Run: `cargo check` from `src-tauri`.

Expected: Rust compile check passes.

- [ ] **Step 2: Run desktop app**

Run: `npm run tauri:dev`

Expected: A Lumora desktop window opens.

If the command blocks while running the dev server, stop after confirming the window starts.
