import { describe, expect, it } from "vitest";
import { addDockItem, createDefaultDockItems } from "./dockItems";
import { loadDockItems, saveDockItems } from "./storage";

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
      label: "项目目录",
      type: "folder",
      target: "D:\\workspace",
    });

    saveDockItems(items, storage);

    expect(loadDockItems(storage).map((item) => item.label)).toEqual(["光枢", "项目目录", "垃圾桶"]);
  });
});
