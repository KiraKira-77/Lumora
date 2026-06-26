import type { DockItem } from "./dockItems";

export type DockLayoutItem = Pick<DockItem, "id" | "pinned"> & {
  left: number;
  right: number;
};

export function dockInsertIndexFromPointer(items: DockLayoutItem[], pointerX: number): number {
  const trashIndex = items.findIndex((item) => item.id === "trash");
  const maxInsertIndex = trashIndex >= 0 ? trashIndex : items.length;
  const firstUserIndex = items.findIndex((item) => !item.pinned);

  if (firstUserIndex < 0) {
    return maxInsertIndex;
  }

  for (let index = firstUserIndex; index < maxInsertIndex; index += 1) {
    const item = items[index];
    const center = item.left + ((item.right - item.left) / 2);
    if (pointerX < center) {
      return index;
    }
  }

  return maxInsertIndex;
}
