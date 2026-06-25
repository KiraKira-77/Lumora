import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import App from "./App";

describe("App", () => {
  it("renders a dock-only surface with only the fixed default icons", () => {
    const html = renderToStaticMarkup(<App surface="dock" />);

    expect(html).toContain('aria-label="Lumora Dock"');
    expect(html).toContain('aria-label="光枢"');
    expect(html).toContain('aria-label="垃圾桶"');
    expect(html).not.toContain("desktop-stage");
    expect(html).not.toContain("Glass Launcher");
    expect(html).not.toContain("微信");
    expect(html).not.toContain("Chrome");
    expect(html).not.toContain("VS Code");
  });

  it("renders a launcher-only surface with empty keyboard slots by default", () => {
    const html = renderToStaticMarkup(<App surface="launcher" />);

    expect(html).toContain('aria-label="Lumora Launcher"');
    expect(html).toContain('aria-label="快捷键 Q"');
    expect(html).toContain('aria-label="关闭启动器"');
    expect(html).not.toContain("shortcut-glyph");
    expect(html).not.toContain("微信");
  });

  it("keeps the Lumora launcher icon before the fixed trash icon", () => {
    const html = renderToStaticMarkup(<App surface="dock" />);

    expect(html.indexOf('aria-label="光枢"')).toBeGreaterThan(-1);
    expect(html.indexOf('aria-label="垃圾桶"')).toBeGreaterThan(-1);
    expect(html.indexOf('aria-label="光枢"')).toBeLessThan(html.indexOf('aria-label="垃圾桶"'));
  });
});
