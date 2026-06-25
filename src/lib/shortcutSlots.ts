import type { DockItem, NewDockItemInput } from "./dockItems";

export type ShortcutKey = string;

export type ShortcutSlotTarget = Pick<DockItem, "id" | "label" | "target" | "glyph" | "tone" | "type" | "iconPath">;

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
          iconPath: targets[index].iconPath,
        }
      : null,
  }));
}

export function bindShortcutSlot(slots: ShortcutSlot[], key: ShortcutKey, input: NewDockItemInput): ShortcutSlot[] {
  const normalizedKey = key.toUpperCase();

  return normalizeShortcutSlots(slots).map((slot) => {
    if (slot.key !== normalizedKey) {
      return slot;
    }

    return {
      key: slot.key,
      target: {
        id: `shortcut_${slot.key}`,
        label: input.label.trim(),
        type: input.type,
        target: input.target.trim(),
        glyph: glyphForLabel(input.label),
        tone: toneForType(input.type),
        iconPath: input.iconPath,
      },
    };
  });
}

export function normalizeShortcutSlots(slots: ShortcutSlot[]): ShortcutSlot[] {
  const byKey = new Map(slots.map((slot) => [slot.key.toUpperCase(), slot]));

  return shortcutKeys.map((key) => {
    const slot = byKey.get(key);
    return slot ? { key, target: slot.target } : { key, target: null };
  });
}

function glyphForLabel(label: string): string {
  return label.trim().slice(0, 1).toUpperCase() || "?";
}

function toneForType(type: NewDockItemInput["type"]): string {
  switch (type) {
    case "app":
      return "code";
    case "folder":
      return "folder";
    case "url":
      return "chrome";
    case "action":
      return "organize";
    case "settings":
      return "settings";
    case "file":
      return "shot";
  }
}
