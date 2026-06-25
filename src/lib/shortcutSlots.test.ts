import { describe, expect, it } from "vitest";
import { createDefaultDockItems } from "./dockItems";
import { createEmptyShortcutSlots, createShortcutSlotsFromDockItems, keyboardRows, shortcutKeys } from "./shortcutSlots";

describe("shortcutSlots", () => {
  it("uses physical keyboard rows for launcher slots", () => {
    expect(keyboardRows).toEqual([
      ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
      ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
      ["A", "S", "D", "F", "G", "H", "J", "K", "L"],
      ["Z", "X", "C", "V", "B", "N", "M"],
    ]);
    expect(shortcutKeys).toHaveLength(36);
  });

  it("creates empty slots with key badges and no targets", () => {
    const slots = createEmptyShortcutSlots();

    expect(slots[0]).toEqual({ key: "1", target: null });
    expect(slots.at(-1)).toEqual({ key: "M", target: null });
    expect(slots.every((slot) => slot.target === null)).toBe(true);
  });

  it("does not bind fixed dock items into launcher shortcut slots", () => {
    const slots = createShortcutSlotsFromDockItems(createDefaultDockItems());

    expect(slots.every((slot) => slot.target === null)).toBe(true);
  });
});
