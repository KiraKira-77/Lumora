import { createDefaultDockItems, sortDockItems, type DockItem } from "./dockItems";
import { createEmptyShortcutSlots, normalizeShortcutSlots, type ShortcutSlot } from "./shortcutSlots";

const dockStorageKey = "lumora.dock.items.v2";
const shortcutStorageKey = "lumora.shortcut.slots.v1";

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

export function loadShortcutSlots(storage: Storage | undefined = getBrowserStorage()): ShortcutSlot[] {
  if (!storage) {
    return createEmptyShortcutSlots();
  }

  const raw = storage.getItem(shortcutStorageKey);
  if (!raw) {
    return createEmptyShortcutSlots();
  }

  try {
    const parsed = JSON.parse(raw) as ShortcutSlot[];
    if (!Array.isArray(parsed)) {
      return createEmptyShortcutSlots();
    }

    return normalizeShortcutSlots(parsed);
  } catch {
    return createEmptyShortcutSlots();
  }
}

export function saveShortcutSlots(slots: ShortcutSlot[], storage: Storage | undefined = getBrowserStorage()): void {
  if (!storage) {
    return;
  }

  storage.setItem(shortcutStorageKey, JSON.stringify(normalizeShortcutSlots(slots)));
}

function normalizeLoadedDockItems(items: DockItem[]): DockItem[] {
  const defaults = createDefaultDockItems();
  const fixedIds = new Set(defaults.map((item) => item.id));
  const userItems = sortDockItems(items).filter((item) => !fixedIds.has(item.id));

  return [...defaults.slice(0, 1), ...userItems, ...defaults.slice(1)].map((item, order) => ({ ...item, order }));
}
