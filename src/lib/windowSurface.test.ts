import { describe, expect, it } from "vitest";
import { surfaceFromWindowLabel } from "./windowSurface";

describe("windowSurface", () => {
  it("maps Tauri window labels to app surfaces", () => {
    expect(surfaceFromWindowLabel("dock")).toBe("dock");
    expect(surfaceFromWindowLabel("launcher")).toBe("launcher");
    expect(surfaceFromWindowLabel("main")).toBe("preview");
    expect(surfaceFromWindowLabel(undefined)).toBe("preview");
  });
});
