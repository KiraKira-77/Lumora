import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { DockSurface } from "./DockSurface";
import { createDefaultDockItems } from "../lib/dockItems";

describe("DockSurface", () => {
  it("renders a dragged app preview with its extracted icon before drop", () => {
    const html = renderToStaticMarkup(
      <DockSurface
        dockItems={createDefaultDockItems()}
        dropPreviewItems={[
          {
            label: "Discord",
            type: "app",
            target: "C:\\Users\\NEX\\Desktop\\Discord.lnk",
            iconPath: "C:\\Users\\NEX\\AppData\\Local\\Lumora\\icons\\discord.png",
          },
        ]}
        isDropHot={true}
        onDragStateChange={() => undefined}
        onDrop={() => undefined}
        onOpen={() => undefined}
        onRemove={() => undefined}
      />,
    );

    expect(html).toContain('aria-label="即将添加 Discord"');
    expect(html).toContain('src="C:\\Users\\NEX\\AppData\\Local\\Lumora\\icons\\discord.png"');
    expect(html.indexOf('aria-label="即将添加 Discord"')).toBeLessThan(html.indexOf('aria-label="垃圾桶"'));
  });
});
