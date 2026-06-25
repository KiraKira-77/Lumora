import { useCallback, useEffect, useState, type DragEvent } from "react";
import { DockSurface } from "./components/DockSurface";
import { LauncherSurface } from "./components/LauncherSurface";
import { addDockItem, type DockItem } from "./lib/dockItems";
import {
  describeNativeTargets,
  hideNativeLauncher,
  listenForNativeDrops,
  openNativeTarget,
  toggleNativeLauncher,
} from "./lib/native";
import { loadDockItems, saveDockItems } from "./lib/storage";
import { getWindowSurface, type WindowSurface } from "./lib/windowSurface";

type AppProps = {
  surface?: WindowSurface;
};

const emptyDropPayload: string[] = [];

export default function App({ surface = getWindowSurface() }: AppProps) {
  const [dockItems, setDockItems] = useState<DockItem[]>(() => loadDockItems());
  const [isDropHot, setIsDropHot] = useState(false);
  const [isPreviewLauncherOpen, setIsPreviewLauncherOpen] = useState(true);

  useEffect(() => {
    saveDockItems(dockItems);
  }, [dockItems]);

  const addTargetsToDock = useCallback(async (targets: string[]) => {
    const cleanTargets = targets.map((target) => target.trim()).filter(Boolean);
    if (cleanTargets.length === 0) {
      return;
    }

    const inputs = await describeNativeTargets(cleanTargets);
    setDockItems((items) => inputs.reduce((next, input) => addDockItem(next, input), items));
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void listenForNativeDrops((paths) => {
      setIsDropHot(false);
      void addTargetsToDock(paths);
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
  }, [addTargetsToDock]);

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
    void addTargetsToDock(targetsFromBrowserDrop(event));
  }

  if (surface === "dock") {
    return (
      <DockSurface
        dockItems={dockItems}
        isDropHot={isDropHot}
        onDragStateChange={setIsDropHot}
        onDrop={handleDrop}
        onOpen={(item) => void handleOpen(item)}
      />
    );
  }

  if (surface === "launcher") {
    return (
      <LauncherSurface
        dockItems={dockItems}
        isDropHot={isDropHot}
        onDragStateChange={setIsDropHot}
        onDrop={handleDrop}
        onOpen={(item) => void handleOpen(item)}
        onClose={() => void hideNativeLauncher()}
      />
    );
  }

  return (
    <main className="preview-surface">
      {isPreviewLauncherOpen ? (
        <LauncherSurface
          dockItems={dockItems}
          isDropHot={isDropHot}
          onDragStateChange={setIsDropHot}
          onDrop={handleDrop}
          onOpen={(item) => void handleOpen(item)}
          onClose={() => setIsPreviewLauncherOpen(false)}
        />
      ) : null}
      <DockSurface
        dockItems={dockItems}
        isDropHot={isDropHot}
        onDragStateChange={setIsDropHot}
        onDrop={handleDrop}
        onOpen={(item) => void handleOpen(item)}
      />
    </main>
  );
}
