import { describe, expect, it } from "vitest";
import {
  addDockItem,
  createDefaultDockItems,
  createDockItemInputFromTarget,
  searchDockItems,
} from "./dockItems";

describe("dock item model", () => {
  it("defaults to only the fixed launcher and trash dock items", () => {
    const items = createDefaultDockItems();

    expect(items).toHaveLength(2);
    expect(items[0]).toMatchObject({
      id: "launcher",
      label: "光枢",
      type: "launcher",
      pinned: true,
    });
    expect(items[1]).toMatchObject({
      id: "trash",
      label: "垃圾桶",
      type: "action",
      pinned: true,
    });
  });

  it("adds a user item before the fixed trash item", () => {
    const items = createDefaultDockItems();
    const next = addDockItem(items, {
      label: "项目目录",
      type: "folder",
      target: "D:\\workspace",
    });

    expect(next).toHaveLength(items.length + 1);
    expect(next[1]).toMatchObject({
      label: "项目目录",
      type: "folder",
      target: "D:\\workspace",
      order: 1,
      pinned: false,
    });
    expect(next[1]?.id).toMatch(/^dock_/);
    expect(next.at(-1)).toMatchObject({
      id: "trash",
      order: 2,
      pinned: true,
    });
  });

  it("searches by label, type, and target", () => {
    const items = addDockItem(createDefaultDockItems(), {
      label: "项目目录",
      type: "folder",
      target: "D:\\workspace\\client-a",
    });

    expect(searchDockItems(items, "项目")).toHaveLength(1);
    expect(searchDockItems(items, "folder").some((item) => item.label === "项目目录")).toBe(true);
    expect(searchDockItems(items, "client-a").map((item) => item.label)).toEqual(["项目目录"]);
  });

  it("creates a URL dock input from dropped text", () => {
    expect(createDockItemInputFromTarget("https://lumora.app/docs")).toEqual({
      label: "lumora.app",
      type: "url",
      target: "https://lumora.app/docs",
    });
  });

  it("creates app, file, and folder dock inputs from dropped paths", () => {
    expect(createDockItemInputFromTarget("C:\\Program Files\\Notion\\Notion.exe")).toMatchObject({
      label: "Notion",
      type: "app",
    });
    expect(createDockItemInputFromTarget("D:\\Projects\\Lumora", "folder")).toEqual({
      label: "Lumora",
      type: "folder",
      target: "D:\\Projects\\Lumora",
    });
    expect(createDockItemInputFromTarget("D:\\Docs\\proposal.final.pdf")).toEqual({
      label: "proposal.final",
      type: "file",
      target: "D:\\Docs\\proposal.final.pdf",
    });
  });
});
