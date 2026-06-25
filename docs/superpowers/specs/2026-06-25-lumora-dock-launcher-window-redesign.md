# Lumora Dock And Launcher Window Redesign

## Problem

The current app opens as a large framed desktop preview. This is the wrong product shape.

Lumora must feel like a Windows desktop layer:

- Launching `lumora.exe` shows only the bottom Dock.
- The desktop wallpaper remains the real Windows desktop, not a CSS background.
- The Dock is not inside a larger window frame.
- The Launcher is hidden by default and appears only from a shortcut or the Dock's Lumora icon.

## Target Shape

Use two Tauri windows.

### Dock Window

- Label: `dock`.
- Starts visible when the app launches.
- Transparent, frameless, always on top.
- Small bottom-centered window, placed above the Windows taskbar.
- Contains the macOS-inspired Dock only.
- The first item is always the Lumora icon.
- Clicking the Lumora icon toggles the Launcher window.
- Users can drag apps, files, folders, or URLs into the Dock.

### Launcher Window

- Label: `launcher`.
- Starts hidden.
- Transparent, frameless, always on top.
- Centered floating glass panel.
- Opens from:
  - global shortcut, initially `Alt+Space` when available;
  - clicking the Lumora icon in the Dock.
- Closes from:
  - `Esc`;
  - clicking the Lumora icon again;
  - optional blur handling after the base behavior is stable.

## Launcher Layout

The Launcher default view is a keyboard-mapped shortcut grid.

Rows:

- `1 2 3 4 5 6 7 8 9 0`
- `Q W E R T Y U I O P`
- `A S D F G H J K L`
- `Z X C V B N M`

Slot behavior:

- Every slot maps to one key.
- Empty slots show an empty keycap with only a small corner key label.
- Filled slots show the target icon in the center.
- Filled slots still show the key label as a corner badge.
- Pressing the mapped key opens the slot target.
- Dragging a target into an empty slot binds it.
- Dragging over a filled slot highlights replacement, but replacement requires explicit drop.

The grid must not use demo labels such as `Folder`, `Widget`, or `App`. The visual vocabulary is keycap plus icon plus corner badge.

## Search

Search remains inside the Launcher, but it is secondary to the shortcut grid.

- Search input is at the top of the Launcher.
- Typing filters Dock items and file results.
- Enter opens the first result.
- Search results appear below or beside the shortcut grid without making the Launcher feel like an admin panel.

## Visual Direction

- No large desktop preview frame.
- No fake wallpaper.
- No visible app background outside the Dock or Launcher glass surfaces.
- No top bar.
- No status strip.
- Dock and Launcher use restrained translucent surfaces.
- Motion should be short, 150 to 220 ms.

## Non-Goals

- Do not implement cloud sync.
- Do not build a settings page in this pass.
- Do not implement full macOS Dock magnification in this pass.
- Do not implement complex desktop organizer UI in this pass.
- Do not preserve the current large all-in-one workbench layout.

## Acceptance Criteria

- Double-clicking `lumora.exe` shows only the Dock.
- The Dock is visually independent, bottom-centered, and not inside a larger frame.
- Clicking the Dock's Lumora icon shows the Launcher.
- Pressing the global shortcut shows the Launcher when the shortcut is available.
- The Launcher is transparent/frameless and visually separate from the Dock.
- The Launcher shortcut grid uses keyboard rows.
- Empty shortcut slots show only key badges.
- Filled shortcut slots show icons plus key badges.
- Existing Dock persistence continues to work.
- Existing target opening continues to work.
