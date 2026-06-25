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
      type: "launcher",
      pinned: true,
    });
    expect(items[1]).toMatchObject({
      id: "trash",
      type: "action",
      pinned: true,
    });
  });

  it("adds a user item with an optional icon before the fixed trash item", () => {
    const items = createDefaultDockItems();
    const next = addDockItem(items, {
      label: "Projects",
      type: "folder",
      target: "D:\\workspace",
      iconPath: "C:\\Users\\NEX\\AppData\\Roaming\\Lumora\\icons\\workspace.png",
    });

    expect(next).toHaveLength(items.length + 1);
    expect(next[1]).toMatchObject({
      label: "Projects",
      type: "folder",
      target: "D:\\workspace",
      iconPath: "C:\\Users\\NEX\\AppData\\Roaming\\Lumora\\icons\\workspace.png",
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
      label: "Projects",
      type: "folder",
      target: "D:\\workspace\\client-a",
    });

    expect(searchDockItems(items, "proj")).toHaveLength(1);
    expect(searchDockItems(items, "folder").some((item) => item.label === "Projects")).toBe(true);
    expect(searchDockItems(items, "client-a").map((item) => item.label)).toEqual(["Projects"]);
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
