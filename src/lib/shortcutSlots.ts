import type { DockItem } from "./dockItems";

export type ShortcutKey = string;

export type ShortcutSlotTarget = Pick<DockItem, "id" | "label" | "target" | "glyph" | "tone" | "type">;

export type ShortcutSlot = {
  key: ShortcutKey;
  target: ShortcutSlotTarget | null;
};

export const keyboardRows = [
  ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
  ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
  ["A", "S", "D", "F", "G", "H", "J", "K", "L"],
  ["Z", "X", "C", "V", "B", "N", "M"],
] as const;

export const shortcutKeys = keyboardRows.flat();

export function createEmptyShortcutSlots(): ShortcutSlot[] {
  return shortcutKeys.map((key) => ({
    key,
    target: null,
  }));
}

export function createShortcutSlotsFromDockItems(items: DockItem[]): ShortcutSlot[] {
  const targets = items.filter((item) => !item.pinned && item.type !== "launcher");

  return shortcutKeys.map((key, index) => ({
    key,
    target: targets[index]
      ? {
          id: targets[index].id,
          label: targets[index].label,
          target: targets[index].target,
          glyph: targets[index].glyph,
          tone: targets[index].tone,
          type: targets[index].type,
        }
      : null,
  }));
}
