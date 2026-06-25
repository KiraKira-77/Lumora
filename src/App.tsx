import { useCallback, useEffect, useRef, useState, type DragEvent } from "react";
import { DockSurface } from "./components/DockSurface";
import { LauncherSurface } from "./components/LauncherSurface";
import { addDockItem, type DockItem } from "./lib/dockItems";
import { bindShortcutSlot, type ShortcutSlotTarget } from "./lib/shortcutSlots";
import {
  describeNativeTargets,
  hideNativeLauncher,
  listenForNativeDrops,
  openNativeTarget,
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
  const dropDestinationRef = useRef<DropDestination>({ type: "dock" });
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

    const inputs = await describeNativeTargets(cleanTargets);
    setDockItems((items) => inputs.reduce((next, input) => addDockItem(next, input), items));
  }, []);

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

  const handleDroppedTargets = useCallback(
    (targets: string[]) => {
      const destination = dropDestinationRef.current;
      if (destination.type === "shortcut") {
        void bindTargetToShortcut(destination.key, targets);
        return;
      }

      void addTargetsToDock(targets);
    },
    [addTargetsToDock, bindTargetToShortcut],
  );

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void listenForNativeDrops((paths) => {
      setIsDropHot(false);
      handleDroppedTargets(paths);
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
  }, [handleDroppedTargets]);

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
    dropDestinationRef.current = { type: "shortcut", key };
    void bindTargetToShortcut(key, targetsFromBrowserDrop(event));
  }

  if (surface === "dock") {
    return (
      <DockSurface
        dockItems={dockItems}
        isDropHot={isDropHot}
        onDragStateChange={(isHot) => {
          setIsDropHot(isHot);
          if (isHot) {
            dropDestinationRef.current = { type: "dock" };
          }
        }}
        onDrop={handleDrop}
        onOpen={(item) => void handleOpen(item)}
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
        isDropHot={isDropHot}
        onDragStateChange={(isHot) => {
          setIsDropHot(isHot);
          if (isHot) {
            dropDestinationRef.current = { type: "dock" };
          }
        }}
        onDrop={handleDrop}
        onOpen={(item) => void handleOpen(item)}
      />
    </main>
  );
}
