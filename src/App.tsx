import { useCallback, useEffect, useRef, useState, type DragEvent } from "react";
import { DockSurface } from "./components/DockSurface";
import { LauncherSurface } from "./components/LauncherSurface";
import { addDockItem, type DockItem, type NewDockItemInput } from "./lib/dockItems";
import { bindShortcutSlot, type ShortcutSlotTarget } from "./lib/shortcutSlots";
import {
  describeNativeTargets,
  hideNativeDesktopFile,
  hideNativeLauncher,
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
  | { type: "dock" }
  | { type: "shortcut"; key: string };

export default function App({ surface = getWindowSurface() }: AppProps) {
  const [dockItems, setDockItems] = useState<DockItem[]>(() => loadDockItems());
  const [shortcutSlots, setShortcutSlots] = useState(() => loadShortcutSlots());
  const [isDropHot, setIsDropHot] = useState(false);
  const [dropPreviewItems, setDropPreviewItems] = useState<NewDockItemInput[]>([]);
  const dropDestinationRef = useRef<DropDestination>({ type: "dock" });
  const dropPreviewRequestRef = useRef(0);
  const [isPreviewLauncherOpen, setIsPreviewLauncherOpen] = useState(true);

  useEffect(() => {
    saveDockItems(dockItems);
  }, [dockItems]);

  useEffect(() => {
    saveShortcutSlots(shortcutSlots);
  }, [shortcutSlots]);

  const addTargetsToDock = useCallback(async (targets: string[]) => {
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

    const inputs = await Promise.all(cleanTargets.map(t => hideNativeDesktopFile(t)));
    setDockItems((items) => inputs.reduce((next, input) => addDockItem(next, input), items));
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

      void addTargetsToDock(targets);
    },
    [addTargetsToDock, bindTargetToShortcut, clearDropPreview],
  );

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void listenForNativeDrops(
      (paths) => {
        setIsDropHot(false);
        handleDroppedTargets(paths);
      },
      {
        onEnter: (paths) => {
          setIsDropHot(true);
          dropDestinationRef.current = { type: "dock" };
          void previewTargetsForDock(paths);
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

  function handleDrop(event: DragEvent<HTMLElement>) {
    event.preventDefault();
    setIsDropHot(false);
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
      await restoreNativeDesktopFile(item.target, item.originalDesktopPath);
    }
    setDockItems((items) => items.filter((i) => i.id !== item.id));
  }

  if (surface === "dock") {
    return (
      <DockSurface
        dockItems={dockItems}
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
      />
    </main>
  );
}
