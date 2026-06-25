import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { createDockItemInputFromTarget, type NewDockItemInput } from "./dockItems";
import { classifyDesktopFile, type DesktopCategory } from "./desktopOrganizer";

export type NativeDesktopCount = {
  category: DesktopCategory;
  label: string;
  count: number;
};

export type NativeDesktopFile = {
  name: string;
  path: string;
  category: DesktopCategory;
  label: string;
};

export type NativeDesktopScanResult = {
  desktop_path: string;
  files: NativeDesktopFile[];
  counts: NativeDesktopCount[];
};

export type NativeMovedDesktopFile = {
  name: string;
  category: DesktopCategory;
  label: string;
  original_path: string;
  current_path: string;
};

export type NativeSkippedDesktopFile = {
  path: string;
  reason: string;
};

export type NativeDesktopOrganizeResult = {
  target_root: string;
  moved_files: NativeMovedDesktopFile[];
  skipped_files: NativeSkippedDesktopFile[];
};

export type NativeDesktopUndoResult = {
  restored_files: NativeMovedDesktopFile[];
  skipped_files: NativeSkippedDesktopFile[];
};

export type NativeFileSearchItem = {
  name: string;
  path: string;
  category: DesktopCategory;
  label: string;
};

export type NativeFileSearchResult = {
  query: string;
  roots: string[];
  total_matches: number;
  files: NativeFileSearchItem[];
};

const sampleFiles = [
  "screenshot.PNG",
  "proposal.pdf",
  "backup.7z",
  "setup.msi",
  "demo.mov",
  "app.tsx",
  "untitled",
];

const categoryLabels: Record<DesktopCategory, string> = {
  Images: "图片",
  Docs: "文档",
  Archives: "压缩包",
  Installers: "安装包",
  Videos: "视频",
  Projects: "项目",
  Inbox: "收纳箱",
};

export async function openNativeTarget(target: string): Promise<string> {
  if (!isTauriRuntime()) {
    return `浏览器预览：${target}`;
  }

  return invoke<string>("open_target", { target });
}

export async function toggleNativeLauncher(): Promise<string> {
  if (!isTauriRuntime()) {
    return "Browser Preview: toggle launcher";
  }

  return invoke<string>("toggle_launcher");
}

export async function hideNativeLauncher(): Promise<string> {
  if (!isTauriRuntime()) {
    return "Browser Preview: hide launcher";
  }

  return invoke<string>("hide_launcher");
}

export async function startNativeWindowDrag(): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }

  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().startDragging();
}

export function filePathToAssetSrc(path: string): string {
  if (!isTauriRuntime()) {
    return path;
  }

  return convertFileSrc(path);
}

export async function describeNativeTargets(targets: string[]): Promise<NewDockItemInput[]> {
  const cleanTargets = targets.map((target) => target.trim()).filter(Boolean);

  if (!isTauriRuntime()) {
    return cleanTargets.map((target) => createDockItemInputFromTarget(target));
  }

  return invoke<NewDockItemInput[]>("describe_targets", { paths: cleanTargets });
}

export async function scanNativeDesktop(): Promise<NativeDesktopScanResult> {
  if (!isTauriRuntime()) {
    const files = sampleFiles.map((name) => {
      const category = classifyDesktopFile(name);
      return {
        name,
        path: `Desktop\\${name}`,
        category,
        label: categoryLabels[category],
      };
    });

    return {
      desktop_path: "Browser Preview Desktop",
      files,
      counts: buildCounts(files),
    };
  }

  return invoke<NativeDesktopScanResult>("scan_desktop");
}

export async function searchNativeFiles(query: string): Promise<NativeFileSearchResult> {
  const cleanQuery = query.trim();

  if (!isTauriRuntime()) {
    const files = sampleFiles
      .filter((name) => name.toLowerCase().includes(cleanQuery.toLowerCase()))
      .map((name) => {
        const category = classifyDesktopFile(name);
        return {
          name,
          path: `Desktop\\${name}`,
          category,
          label: categoryLabels[category],
        };
      });

    return {
      query: cleanQuery,
      roots: ["Browser Preview Desktop"],
      total_matches: files.length,
      files,
    };
  }

  return invoke<NativeFileSearchResult>("search_files", { query: cleanQuery });
}

export async function organizeNativeDesktop(paths: string[]): Promise<NativeDesktopOrganizeResult> {
  if (!isTauriRuntime()) {
    const scan = await scanNativeDesktop();
    const selected = paths.length > 0 ? scan.files.filter((file) => paths.includes(file.path)) : scan.files;
    return {
      target_root: "Browser Preview Desktop\\Lumora整理",
      moved_files: selected.map((file) => ({
        name: file.name,
        category: file.category,
        label: file.label,
        original_path: file.path,
        current_path: `Desktop\\Lumora整理\\${file.label}\\${file.name}`,
      })),
      skipped_files: [],
    };
  }

  return invoke<NativeDesktopOrganizeResult>("organize_desktop", { paths });
}

export async function undoNativeDesktopOrganize(
  movedFiles: NativeMovedDesktopFile[],
): Promise<NativeDesktopUndoResult> {
  if (!isTauriRuntime()) {
    return {
      restored_files: movedFiles,
      skipped_files: [],
    };
  }

  return invoke<NativeDesktopUndoResult>("undo_desktop_organize", { movedFiles });
}

export async function hideNativeDesktopFile(pathStr: string): Promise<NewDockItemInput> {
  if (!isTauriRuntime()) {
    return {
      label: "Mock",
      type: "file",
      target: pathStr,
    };
  }
  return invoke<NewDockItemInput>("hide_desktop_file", { pathStr });
}

export async function restoreNativeDesktopFile(currentPath: string, originalPath: string): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }
  return invoke<void>("restore_desktop_file", { currentPath, originalPath });
}

type NativeDropOptions = {
  onEnter?: (paths: string[]) => void;
  onLeave?: () => void;
};

export async function listenForNativeDrops(
  onDrop: (paths: string[]) => void,
  options: NativeDropOptions = {},
): Promise<() => void> {
  if (!isTauriRuntime()) {
    return () => {};
  }

  const { getCurrentWebview } = await import("@tauri-apps/api/webview");
  return getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === "enter" && event.payload.paths.length > 0) {
      options.onEnter?.(event.payload.paths);
      return;
    }

    if (event.payload.type === "drop") {
      if (event.payload.paths.length > 0) {
        onDrop(event.payload.paths);
      }
      options.onLeave?.();
      return;
    }

    if (event.payload.type === "leave") {
      options.onLeave?.();
    }
  });
}

export async function updateDockWindowBounds(dockWidth: number, dockHeight: number): Promise<void> {
  if (!isTauriRuntime()) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("update_dock_bounds", { width: dockWidth, height: dockHeight });
  } catch (err: any) {
    alert("Error updating window size: " + (err.message || String(err)));
  }
}

export async function listenForLauncherFocus(onFocus: () => void): Promise<() => void> {
  if (!isTauriRuntime()) {
    return () => {};
  }

  const { listen } = await import("@tauri-apps/api/event");
  return listen("lumora://launcher-focus", () => onFocus());
}

function buildCounts(files: NativeDesktopFile[]): NativeDesktopCount[] {
  return Object.entries(categoryLabels).map(([category, label]) => ({
    category: category as DesktopCategory,
    label,
    count: files.filter((file) => file.category === category).length,
  }));
}

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
