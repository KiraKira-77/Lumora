import { createDefaultDockItems, sortDockItems, type DockItem } from "./dockItems";

const dockStorageKey = "lumora.dock.items.v2";

function getBrowserStorage(): Storage | undefined {
  if (typeof window === "undefined") {
    return undefined;
  }

  return window.localStorage;
}

export function loadDockItems(storage: Storage | undefined = getBrowserStorage()): DockItem[] {
  if (!storage) {
    return createDefaultDockItems();
  }

  const raw = storage.getItem(dockStorageKey);

  if (!raw) {
    return createDefaultDockItems();
  }

  try {
    const parsed = JSON.parse(raw) as DockItem[];
    if (!Array.isArray(parsed)) {
      return createDefaultDockItems();
    }

    return normalizeLoadedDockItems(parsed);
  } catch {
    return createDefaultDockItems();
  }
}

export function saveDockItems(items: DockItem[], storage: Storage | undefined = getBrowserStorage()): void {
  if (!storage) {
    return;
  }

  storage.setItem(dockStorageKey, JSON.stringify(sortDockItems(items)));
}

function normalizeLoadedDockItems(items: DockItem[]): DockItem[] {
  const defaults = createDefaultDockItems();
  const fixedIds = new Set(defaults.map((item) => item.id));
  const userItems = sortDockItems(items).filter((item) => !fixedIds.has(item.id));

  return [...defaults.slice(0, 1), ...userItems, ...defaults.slice(1)].map((item, order) => ({ ...item, order }));
}
