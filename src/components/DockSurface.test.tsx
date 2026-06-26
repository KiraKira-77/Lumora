import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { DockSurface } from "./DockSurface";
import { addDockItem, createDefaultDockItems } from "../lib/dockItems";

function buttonMarkupByLabel(html: string, label: string): string {
  return html.match(new RegExp(`<button[^>]*aria-label="${label}"[\\s\\S]*?</button>`))?.[0] ?? "";
}

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

  it("marks user dock items as draggable without making fixed icons draggable", () => {
    const dockItems = addDockItem(createDefaultDockItems(), {
      label: "Projects",
      type: "folder",
      target: "D:\\Projects",
    });

    const html = renderToStaticMarkup(
      <DockSurface
        dockItems={dockItems}
        isDropHot={false}
        onDragStateChange={() => undefined}
        onDrop={() => undefined}
        onOpen={() => undefined}
        onRemove={() => undefined}
      />,
    );

    expect(html).toContain('aria-label="Projects"');
    expect(html).toContain('draggable="true"');
    expect(html).not.toContain('aria-label="光枢" draggable="true"');
    expect(html).not.toContain('aria-label="垃圾桶" draggable="true"');
  });

  it("renders a subtle running indicator without attention background when there is no message attention", () => {
    const dockItems = addDockItem(createDefaultDockItems(), {
      label: "WeChat",
      type: "app",
      target: "C:\\Program Files\\Tencent\\WeChat\\WeChat.exe",
    });

    const html = renderToStaticMarkup(
      <DockSurface
        dockItems={dockItems}
        dockItemStatuses={{
          "C:\\Program Files\\Tencent\\WeChat\\WeChat.exe": {
            isRunning: true,
            needsAttention: false,
            attentionSequence: 0,
          },
        }}
        isDropHot={false}
        onDragStateChange={() => undefined}
        onDrop={() => undefined}
        onOpen={() => undefined}
        onRemove={() => undefined}
      />,
    );

    const weChatButton = buttonMarkupByLabel(html, "WeChat");

    expect(weChatButton).toContain('aria-label="WeChat"');
    expect(weChatButton).toContain('class="dock-running-indicator"');
    expect(weChatButton).not.toContain("is-attention");
    expect(weChatButton).not.toContain("is-bouncing");
    expect(weChatButton).not.toContain("<i aria-hidden=\"true\"></i>");
  });

  it("renders attention as an orange cue without unread counts or red dots", () => {
    const dockItems = addDockItem(createDefaultDockItems(), {
      label: "WeChat",
      type: "app",
      target: "C:\\Program Files\\Tencent\\WeChat\\WeChat.exe",
    });

    const html = renderToStaticMarkup(
      <DockSurface
        dockItems={dockItems}
        dockItemStatuses={{
          "C:\\Program Files\\Tencent\\WeChat\\WeChat.exe": {
            isRunning: true,
            needsAttention: true,
            attentionSequence: 1,
          },
        }}
        isDropHot={false}
        onDragStateChange={() => undefined}
        onDrop={() => undefined}
        onOpen={() => undefined}
        onRemove={() => undefined}
      />,
    );

    const weChatButton = buttonMarkupByLabel(html, "WeChat");

    expect(weChatButton).toContain('aria-label="WeChat"');
    expect(weChatButton).toContain("is-attention");
    expect(weChatButton).toContain("is-bouncing");
    expect(weChatButton).toContain('class="dock-running-indicator"');
    expect(weChatButton).not.toContain("dock-attention-dot");
    expect(weChatButton).not.toContain("dock-unread-count");
  });

  it("keeps the orange attention underline when running status lags behind attention", () => {
    const dockItems = addDockItem(createDefaultDockItems(), {
      label: "WeChat",
      type: "app",
      target: "C:\\Program Files\\Tencent\\WeChat\\WeChat.exe",
    });

    const html = renderToStaticMarkup(
      <DockSurface
        dockItems={dockItems}
        dockItemStatuses={{
          "C:\\Program Files\\Tencent\\WeChat\\WeChat.exe": {
            isRunning: false,
            needsAttention: true,
            attentionSequence: 2,
          },
        }}
        isDropHot={false}
        onDragStateChange={() => undefined}
        onDrop={() => undefined}
        onOpen={() => undefined}
        onRemove={() => undefined}
      />,
    );

    const weChatButton = buttonMarkupByLabel(html, "WeChat");

    expect(weChatButton).toContain("is-attention");
    expect(weChatButton).toContain('class="dock-running-indicator"');
  });
});
