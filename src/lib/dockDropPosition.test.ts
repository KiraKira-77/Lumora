import { describe, expect, it } from "vitest";
import { dockInsertIndexFromPointer, type DockLayoutItem } from "./dockDropPosition";

describe("dockInsertIndexFromPointer", () => {
  const layout: DockLayoutItem[] = [
    { id: "launcher", pinned: true, left: 0, right: 50 },
    { id: "projects", pinned: false, left: 60, right: 110 },
    { id: "wechat", pinned: false, left: 120, right: 170 },
    { id: "trash", pinned: true, left: 180, right: 230 },
  ];

  it("inserts before the first user item when dropped near the left user area", () => {
    expect(dockInsertIndexFromPointer(layout, 70)).toBe(1);
  });

  it("inserts between user items based on icon centers", () => {
    expect(dockInsertIndexFromPointer(layout, 115)).toBe(2);
  });

  it("inserts before trash when dropped after all user items", () => {
    expect(dockInsertIndexFromPointer(layout, 175)).toBe(3);
  });
});
