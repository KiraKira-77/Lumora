import { describe, expect, it } from "vitest";
import { addDockItem, createDefaultDockItems } from "./dockItems";
import { bindShortcutSlot, createEmptyShortcutSlots } from "./shortcutSlots";
import { loadDockItems, loadShortcutSlots, saveDockItems, saveShortcutSlots } from "./storage";

class MemoryStorage implements Storage {
  private values = new Map<string, string>();
  length = 0;

  clear(): void {
    this.values.clear();
    this.length = 0;
  }

  getItem(key: string): string | null {
    return this.values.get(key) ?? null;
  }

  key(index: number): string | null {
    return [...this.values.keys()][index] ?? null;
  }

  removeItem(key: string): void {
    this.values.delete(key);
    this.length = this.values.size;
  }

  setItem(key: string, value: string): void {
    this.values.set(key, value);
    this.length = this.values.size;
  }
}

describe("dock storage", () => {
  it("loads default dock items when storage is empty", () => {
    const items = loadDockItems(new MemoryStorage());

    expect(items.map((item) => item.id)).toEqual(["launcher", "trash"]);
    expect(items).toHaveLength(createDefaultDockItems().length);
  });

  it("saves and reloads user-added dock items", () => {
    const storage = new MemoryStorage();
    const items = addDockItem(createDefaultDockItems(), {
      label: "Projects",
      type: "folder",
      target: "D:\\workspace",
      iconPath: "C:\\Users\\NEX\\AppData\\Roaming\\Lumora\\icons\\workspace.png",
    });

    saveDockItems(items, storage);

    expect(loadDockItems(storage).map((item) => item.label)).toEqual(["光枢", "Projects", "垃圾桶"]);
    expect(loadDockItems(storage)[1]?.iconPath).toBe("C:\\Users\\NEX\\AppData\\Roaming\\Lumora\\icons\\workspace.png");
  });
});

describe("shortcut storage", () => {
  it("loads empty shortcut slots when storage is empty", () => {
    const slots = loadShortcutSlots(new MemoryStorage());

    expect(slots).toHaveLength(createEmptyShortcutSlots().length);
    expect(slots.every((slot) => slot.target === null)).toBe(true);
  });

  it("saves and reloads shortcut slot bindings", () => {
    const storage = new MemoryStorage();
    const slots = bindShortcutSlot(createEmptyShortcutSlots(), "Q", {
      label: "Notion",
      type: "app",
      target: "C:\\Program Files\\Notion\\Notion.exe",
      iconPath: "C:\\Users\\NEX\\AppData\\Roaming\\Lumora\\icons\\notion.png",
    });

    saveShortcutSlots(slots, storage);

    expect(loadShortcutSlots(storage).find((slot) => slot.key === "Q")?.target).toMatchObject({
      label: "Notion",
      target: "C:\\Program Files\\Notion\\Notion.exe",
      iconPath: "C:\\Users\\NEX\\AppData\\Roaming\\Lumora\\icons\\notion.png",
    });
  });
});
