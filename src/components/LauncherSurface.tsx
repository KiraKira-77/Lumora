import type { DragEvent, MouseEvent } from "react";
import { useEffect, useMemo, useState } from "react";
import type { DockItem } from "../lib/dockItems";
import { searchDockItems } from "../lib/dockItems";
import { keyboardRows, type ShortcutSlot, type ShortcutSlotTarget } from "../lib/shortcutSlots";
import {
  hideNativeLauncher,
  filePathToAssetSrc,
  openNativeTarget,
  searchNativeFiles,
  startNativeWindowDrag,
  type NativeFileSearchItem,
} from "../lib/native";

type LauncherSurfaceProps = {
  dockItems: DockItem[];
  shortcutSlots: ShortcutSlot[];
  isDropHot: boolean;
  onDragStateChange: (isHot: boolean) => void;
  onDrop: (event: DragEvent<HTMLElement>) => void;
  onShortcutDragEnter: (key: string) => void;
  onShortcutDrop: (key: string, event: DragEvent<HTMLElement>) => void;
  onOpenTarget: (target: ShortcutSlotTarget) => void;
  onClose: () => void;
};

function isEditableTarget(target: EventTarget | null): boolean {
  return target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement;
}

function normalizedShortcutKey(event: KeyboardEvent): string {
  if (event.key.length !== 1) {
    return "";
  }

  return event.key.toUpperCase();
}

export function LauncherSurface({
  dockItems,
  shortcutSlots,
  isDropHot,
  onDragStateChange,
  onDrop,
  onShortcutDragEnter,
  onShortcutDrop,
  onOpenTarget,
  onClose,
}: LauncherSurfaceProps) {
  const [query, setQuery] = useState("");
  const [fileResults, setFileResults] = useState<NativeFileSearchItem[]>([]);
  const dockResults = useMemo(
    () => searchDockItems(dockItems, query).filter((item) => item.type !== "launcher" && item.id !== "trash"),
    [dockItems, query],
  );

  useEffect(() => {
    const cleanQuery = query.trim();
    if (cleanQuery.length < 2) {
      setFileResults([]);
      return;
    }

    let disposed = false;
    const timer = window.setTimeout(() => {
      void searchNativeFiles(cleanQuery)
        .then((result) => {
          if (!disposed) {
            setFileResults(result.files);
          }
        })
        .catch(() => {
          if (!disposed) {
            setFileResults([]);
          }
        });
    }, 180);

    return () => {
      disposed = true;
      window.clearTimeout(timer);
    };
  }, [query]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        void hideNativeLauncher();
        return;
      }

      if (isEditableTarget(event.target)) {
        return;
      }

      const key = normalizedShortcutKey(event);
      if (!key) {
        return;
      }

      const slot = shortcutSlots.find((item) => item.key === key);
      if (!slot?.target) {
        return;
      }

      event.preventDefault();
      onOpenTarget(slot.target);
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onOpenTarget, shortcutSlots]);

  function renderSlot(slot: ShortcutSlot) {
    return (
      <button
        className={`shortcut-slot ${slot.target ? "is-filled" : "is-empty"} dock-${slot.target?.tone ?? "empty"}`}
        key={slot.key}
        aria-label={`快捷键 ${slot.key}${slot.target ? ` ${slot.target.label}` : ""}`}
        title={slot.target?.label ?? `快捷键 ${slot.key}`}
        onDragEnter={(event) => {
          event.stopPropagation();
          onDragStateChange(true);
          onShortcutDragEnter(slot.key);
        }}
        onDragOver={(event) => {
          event.preventDefault();
          event.stopPropagation();
        }}
        onDrop={(event) => {
          event.stopPropagation();
          onShortcutDrop(slot.key, event);
        }}
        onClick={() => {
          if (slot.target) {
            onOpenTarget(slot.target);
          }
        }}
      >
        <span className="shortcut-badge">{slot.key}</span>
        {slot.target?.iconPath ? (
          <img className="shortcut-icon" src={filePathToAssetSrc(slot.target.iconPath)} alt="" aria-hidden="true" />
        ) : slot.target ? (
          <span className="shortcut-glyph">{slot.target.glyph}</span>
        ) : null}
      </button>
    );
  }

  function handleLauncherDragStart(event: MouseEvent<HTMLElement>) {
    if (event.button !== 0 || isEditableTarget(event.target)) {
      return;
    }

    void startNativeWindowDrag();
  }

  return (
    <main
      className="launcher-window-surface"
      onDragEnter={() => onDragStateChange(true)}
      onDragOver={(event) => event.preventDefault()}
      onDragLeave={() => onDragStateChange(false)}
      onDrop={onDrop}
    >
      <section className={`lumora-launcher ${isDropHot ? "is-drop-hot" : ""}`} aria-label="Lumora Launcher">
        <div className="launcher-chrome" data-tauri-drag-region onMouseDown={handleLauncherDragStart}>
          <button
            className="launcher-close"
            type="button"
            aria-label="关闭启动器"
            title="关闭启动器"
            onMouseDown={(event) => event.stopPropagation()}
            onClick={onClose}
          >
            <span aria-hidden="true" />
          </button>
          <div className="launcher-drag-handle" aria-hidden="true" />
        </div>

        <input
          className="launcher-search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="搜索应用、文件、文件夹"
          aria-label="搜索应用、文件、文件夹"
        />

        <div className="shortcut-board" aria-label="键盘快捷槽">
          {keyboardRows.map((row) => (
            <div className="shortcut-row" key={row.join("")}>
              {row.map((key) => renderSlot(shortcutSlots.find((slot) => slot.key === key) ?? { key, target: null }))}
            </div>
          ))}
        </div>

        {query.trim() ? (
          <div className="launcher-results" aria-label="搜索结果">
            {dockResults.slice(0, 4).map((item) => (
              <button className="launcher-result" key={item.id} onClick={() => onOpenTarget(item)}>
                {item.iconPath ? (
                  <img className="result-icon" src={filePathToAssetSrc(item.iconPath)} alt="" aria-hidden="true" />
                ) : (
                  <span className={`result-glyph dock-${item.tone}`}>{item.glyph}</span>
                )}
                <span>{item.label}</span>
              </button>
            ))}
            {fileResults.slice(0, 4).map((file) => (
              <button
                className="launcher-result"
                key={file.path}
                onClick={() => void openNativeTarget(file.path)}
              >
                <span className="result-glyph dock-folder">F</span>
                <span>{file.name}</span>
              </button>
            ))}
          </div>
        ) : null}
      </section>
    </main>
  );
}
