import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

describe("Tauri window config", () => {
  it("defines separate frameless transparent dock and launcher windows", () => {
    const config = JSON.parse(readFileSync(join(process.cwd(), "src-tauri", "tauri.conf.json"), "utf8"));
    const windows = config.app.windows;
    const dockWindow = windows.find((window: { label: string }) => window.label === "dock");
    const launcherWindow = windows.find((window: { label: string }) => window.label === "launcher");

    expect(windows.some((window: { label: string }) => window.label === "main")).toBe(false);
    expect(dockWindow).toMatchObject({
      decorations: false,
      transparent: true,
      shadow: false,
      alwaysOnTop: true,
      resizable: false,
      visible: false,
    });
    expect(launcherWindow).toMatchObject({
      decorations: false,
      transparent: true,
      shadow: false,
      alwaysOnTop: true,
      resizable: false,
      visible: false,
    });
  });
});
