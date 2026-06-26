import { useCallback, useEffect, useRef, useState, type DragEvent } from "react";
import { DockSurface, type DockInsertIndexResolver, type DockRuntimeStatus } from "./components/DockSurface";
import { LauncherSurface } from "./components/LauncherSurface";
import { addDockItemAt, reorderDockItem, type DockItem, type NewDockItemInput } from "./lib/dockItems";
import { bindShortcutSlot, type ShortcutSlotTarget } from "./lib/shortcutSlots";
import {
  clearNativeDockItemAttention,
  describeNativeTargets,
  getNativeDockItemStatuses,
  hideNativeDesktopFile,
  hideNativeLauncher,
  listenForDockAttentionChanges,
  listenForNativeDrops,
  openNativeTarget,
  restoreNativeDesktopFile,
  toggleNativeLauncher,
} from "./lib/native";
import { loadDockItems, loadShortcutSlots, saveDockItems, saveShortcutSlots } from "./lib/storage";
import { getWindowSurface, type WindowSurface } from "./lib/windowSurface";

type AppProps = {
  surface?: WindowSurface;
};

const emptyDropPayload: string[] = [];

type DropDestination =
  | { type: "dock"; insertIndex?: number }
  | { type: "shortcut"; key: string };

export default function App({ surface = getWindowSurface() }: AppProps) {
  const [dockItems, setDockItems] = useState<DockItem[]>(() => loadDockItems());
  const [shortcutSlots, setShortcutSlots] = useState(() => loadShortcutSlots());
  const [isDropHot, setIsDropHot] = useState(false);
  const [dropPreviewItems, setDropPreviewItems] = useState<NewDockItemInput[]>([]);
  const [dockItemStatuses, setDockItemStatuses] = useState<Record<string, DockRuntimeStatus>>({});
  const dropDestinationRef = useRef<DropDestination>({ type: "dock" });
  const dropPreviewRequestRef = useRef(0);
  const dockInsertIndexResolverRef = useRef<DockInsertIndexResolver | null>(null);
  const [isPreviewLauncherOpen, setIsPreviewLauncherOpen] = useState(true);

  useEffect(() => {
    saveDockItems(dockItems);
  }, [dockItems]);

  useEffect(() => {
    saveShortcutSlots(shortcutSlots);
  }, [shortcutSlots]);

  const refreshDockItemStatuses = useCallback(async () => {
    const targets = dockItems.map((item) => item.target).filter(Boolean);
    if (targets.length === 0) {
      setDockItemStatuses({});
      return;
    }

    const statuses = await getNativeDockItemStatuses(targets);
    setDockItemStatuses(
      Object.fromEntries(
        statuses.map((status) => [
          status.target,
          {
            isRunning: status.isRunning,
            needsAttention: status.needsAttention,
            attentionSequence: status.attentionSequence,
          },
        ]),
      ),
    );
  }, [dockItems]);

  useEffect(() => {
    let disposed = false;

    async function refreshIfMounted() {
      try {
        await refreshDockItemStatuses();
      } catch {
        if (!disposed) {
          setDockItemStatuses({});
        }
      }
    }

    void refreshIfMounted();
    const intervalId = window.setInterval(() => {
      void refreshIfMounted();
    }, 2000);

    return () => {
      disposed = true;
      window.clearInterval(intervalId);
    };
  }, [refreshDockItemStatuses]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void listenForDockAttentionChanges(() => {
      void refreshDockItemStatuses();
    })
      .then((nextUnlisten) => {
        if (disposed) {
          nextUnlisten();
          return;
        }
        unlisten = nextUnlisten;
      })
      .catch(() => undefined);

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [refreshDockItemStatuses]);

  const addTargetsToDock = useCallback(async (targets: string[], insertIndex?: number) => {
    const cleanTargets = targets.map((target) => target.trim()).filter(Boolean);
    if (cleanTargets.length === 0) {
      return;
    }

    const duplicates = cleanTargets.filter(t => {
      const tLower = t.toLowerCase();
      return dockItems.some(item => 
        item.target.toLowerCase() === tLower || 
        item.target.toLowerCase().endsWith(tLower) ||
        item.label.toLowerCase() === tLower ||
        (item.originalDesktopPath && item.originalDesktopPath.toLowerCase().endsWith(tLower))
      );
    });

    if (duplicates.length > 0) {
      alert(`已经存在于 Dock 中，无需重复添加:\n${duplicates.join("\n")}`);
      return;
    }

    try {
      const inputs = await Promise.all(cleanTargets.map(t => hideNativeDesktopFile(t)));
      setDockItems((items) => {
        const fallbackIndex = items.findIndex((item) => item.id === "trash");
        const firstIndex = insertIndex ?? (fallbackIndex >= 0 ? fallbackIndex : items.length);
        return inputs.reduce((next, input, offset) => addDockItemAt(next, input, firstIndex + offset), items);
      });
    } catch (error) {
      alert(`添加到 Dock 失败：${error instanceof Error ? error.message : String(error)}`);
    }
  }, [dockItems]);

  const bindTargetToShortcut = useCallback(async (key: string, targets: string[]) => {
    const cleanTargets = targets.map((target) => target.trim()).filter(Boolean);
    if (cleanTargets.length === 0) {
      return;
    }

    const [input] = await describeNativeTargets([cleanTargets[0]]);
    if (!input) {
      return;
    }

    setShortcutSlots((slots) => bindShortcutSlot(slots, key, input));
  }, []);

  const clearDropPreview = useCallback(() => {
    dropPreviewRequestRef.current += 1;
    setDropPreviewItems([]);
  }, []);

  const previewTargetsForDock = useCallback(async (targets: string[]) => {
    const cleanTargets = targets.map((target) => target.trim()).filter(Boolean);
    if (cleanTargets.length === 0) {
      clearDropPreview();
      return;
    }

    const requestId = dropPreviewRequestRef.current + 1;
    dropPreviewRequestRef.current = requestId;

    try {
      const inputs = await describeNativeTargets(cleanTargets.slice(0, 4));
      if (dropPreviewRequestRef.current === requestId) {
        setDropPreviewItems(inputs);
      }
    } catch {
      if (dropPreviewRequestRef.current === requestId) {
        setDropPreviewItems([]);
      }
    }
  }, [clearDropPreview]);

  const handleDroppedTargets = useCallback(
    (targets: string[]) => {
      clearDropPreview();
      const destination = dropDestinationRef.current;
      if (destination.type === "shortcut") {
        void bindTargetToShortcut(destination.key, targets);
        return;
      }

      void addTargetsToDock(targets, destination.insertIndex);
    },
    [addTargetsToDock, bindTargetToShortcut, clearDropPreview],
  );

  const handleDockInsertIndexResolverChange = useCallback((resolver: DockInsertIndexResolver | null) => {
    dockInsertIndexResolverRef.current = resolver;
  }, []);

  function updateDockDropDestinationFromPosition(position?: { x: number; y: number }) {
    const insertIndex = position ? dockInsertIndexResolverRef.current?.(position.x) : undefined;
    dropDestinationRef.current = insertIndex === undefined ? { type: "dock" } : { type: "dock", insertIndex };
  }

  function clearDockAttention(target: string) {
    setDockItemStatuses((statuses) => {
      const status = statuses[target];
      if (!status?.needsAttention) {
        return statuses;
      }

      return {
        ...statuses,
        [target]: {
          ...status,
          needsAttention: false,
          attentionSequence: 0,
        },
      };
    });
    void clearNativeDockItemAttention(target);
  }

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void listenForNativeDrops(
      (paths, position) => {
        setIsDropHot(false);
        updateDockDropDestinationFromPosition(position);
        handleDroppedTargets(paths);
      },
      {
        onEnter: (paths, position) => {
          setIsDropHot(true);
          updateDockDropDestinationFromPosition(position);
          void previewTargetsForDock(paths);
        },
        onOver: (position) => {
          updateDockDropDestinationFromPosition(position);
        },
        onLeave: () => {
          setIsDropHot(false);
          clearDropPreview();
        },
      },
    )
      .then((nextUnlisten) => {
        if (disposed) {
          nextUnlisten();
          return;
        }
        unlisten = nextUnlisten;
      })
      .catch(() => undefined);

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [clearDropPreview, handleDroppedTargets, previewTargetsForDock]);

  async function handleOpen(item: DockItem) {
    if (item.type === "launcher") {
      if (surface === "preview") {
        setIsPreviewLauncherOpen((isOpen) => !isOpen);
        return;
      }

      await toggleNativeLauncher();
      return;
    }

    clearDockAttention(item.target);
    await openNativeTarget(item.target);
  }

  async function handleOpenTarget(target: ShortcutSlotTarget) {
    await openNativeTarget(target.target);
  }

  function targetsFromBrowserDrop(event: DragEvent<HTMLElement>): string[] {
    const fileTargets = Array.from(event.dataTransfer.files).map((file) => {
      const maybePath = file as File & { path?: string };
      return maybePath.path || file.name;
    });
    const uriTargets = event.dataTransfer
      .getData("text/uri-list")
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter((line) => line && !line.startsWith("#"));
    const textTarget = event.dataTransfer.getData("text/plain").trim();

    if (fileTargets.length > 0) {
      return fileTargets;
    }

    if (uriTargets.length > 0) {
      return uriTargets;
    }

    return textTarget ? [textTarget] : emptyDropPayload;
  }

  function handleDrop(event: DragEvent<HTMLElement>, insertIndex?: number) {
    event.preventDefault();
    setIsDropHot(false);
    if (insertIndex !== undefined) {
      dropDestinationRef.current = { type: "dock", insertIndex };
    }
    handleDroppedTargets(targetsFromBrowserDrop(event));
  }

  function handleShortcutDrop(key: string, event: DragEvent<HTMLElement>) {
    event.preventDefault();
    setIsDropHot(false);
    clearDropPreview();
    dropDestinationRef.current = { type: "shortcut", key };
    void bindTargetToShortcut(key, targetsFromBrowserDrop(event));
  }

  async function handleRemove(item: DockItem) {
    if (item.originalDesktopPath && item.target) {
      try {
        await restoreNativeDesktopFile(item.target, item.originalDesktopPath);
      } catch (error) {
        alert(`移出 Dock 失败，桌面图标未还原：${error instanceof Error ? error.message : String(error)}`);
        return;
      }
    }
    setDockItems((items) => items.filter((i) => i.id !== item.id));
  }

  function handleDockReorder(draggedId: string, targetId: string) {
    setDockItems((items) => reorderDockItem(items, draggedId, targetId));
  }

  if (surface === "dock") {
    return (
      <DockSurface
        dockItems={dockItems}
        dockItemStatuses={dockItemStatuses}
        dropPreviewItems={dropPreviewItems}
        isDropHot={isDropHot}
        onDragStateChange={(isHot) => {
          setIsDropHot(isHot);
          if (isHot) {
            dropDestinationRef.current = { type: "dock" };
          } else {
            clearDropPreview();
          }
        }}
        onDrop={handleDrop}
        onOpen={(item) => void handleOpen(item)}
        onRemove={(item) => void handleRemove(item)}
        onInsertIndexResolverChange={handleDockInsertIndexResolverChange}
        onReorder={handleDockReorder}
      />
    );
  }

  if (surface === "launcher") {
    return (
      <LauncherSurface
        dockItems={dockItems}
        shortcutSlots={shortcutSlots}
        isDropHot={isDropHot}
        onDragStateChange={(isHot) => {
          setIsDropHot(isHot);
          if (isHot) {
            dropDestinationRef.current = { type: "dock" };
          }
        }}
        onDrop={handleDrop}
        onShortcutDragEnter={(key) => {
          dropDestinationRef.current = { type: "shortcut", key };
        }}
        onShortcutDrop={handleShortcutDrop}
        onOpenTarget={(target) => void handleOpenTarget(target)}
        onClose={() => void hideNativeLauncher()}
      />
    );
  }

  return (
    <main className="preview-surface">
      {isPreviewLauncherOpen ? (
        <LauncherSurface
          dockItems={dockItems}
          shortcutSlots={shortcutSlots}
          isDropHot={isDropHot}
          onDragStateChange={(isHot) => {
            setIsDropHot(isHot);
            if (isHot) {
              dropDestinationRef.current = { type: "dock" };
            }
          }}
          onDrop={handleDrop}
          onShortcutDragEnter={(key) => {
            dropDestinationRef.current = { type: "shortcut", key };
          }}
          onShortcutDrop={handleShortcutDrop}
          onOpenTarget={(target) => void handleOpenTarget(target)}
          onClose={() => setIsPreviewLauncherOpen(false)}
        />
      ) : null}
      <DockSurface
        dockItems={dockItems}
        dockItemStatuses={dockItemStatuses}
        dropPreviewItems={dropPreviewItems}
        isDropHot={isDropHot}
        onDragStateChange={(isHot) => {
          setIsDropHot(isHot);
          if (isHot) {
            dropDestinationRef.current = { type: "dock" };
          } else {
            clearDropPreview();
          }
        }}
        onDrop={handleDrop}
        onOpen={(item) => void handleOpen(item)}
        onRemove={(item) => void handleRemove(item)}
        onInsertIndexResolverChange={handleDockInsertIndexResolverChange}
        onReorder={handleDockReorder}
      />
    </main>
  );
}
