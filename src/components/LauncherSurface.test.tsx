import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { LauncherSurface } from "./LauncherSurface";
import { bindShortcutSlot, createEmptyShortcutSlots } from "../lib/shortcutSlots";

describe("LauncherSurface", () => {
  it("renders a bound shortcut slot with its app icon", () => {
    const shortcutSlots = bindShortcutSlot(createEmptyShortcutSlots(), "Q", {
      label: "Notion",
      type: "app",
      target: "C:\\Program Files\\Notion\\Notion.exe",
      iconPath: "C:\\Users\\NEX\\AppData\\Roaming\\Lumora\\icons\\notion.png",
    });

    const html = renderToStaticMarkup(
      <LauncherSurface
        dockItems={[]}
        shortcutSlots={shortcutSlots}
        isDropHot={false}
        onDragStateChange={() => undefined}
        onDrop={() => undefined}
        onShortcutDragEnter={() => undefined}
        onShortcutDrop={() => undefined}
        onOpenTarget={() => undefined}
        onClose={() => undefined}
      />,
    );

    expect(html).toContain('aria-label="快捷键 Q Notion"');
    expect(html).toContain('src="C:\\Users\\NEX\\AppData\\Roaming\\Lumora\\icons\\notion.png"');
  });
});
