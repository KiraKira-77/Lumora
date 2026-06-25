import { describe, expect, it } from "vitest";
import { buildOrganizerPlan, classifyDesktopFile } from "./desktopOrganizer";

describe("classifyDesktopFile", () => {
  it("classifies common image files", () => {
    expect(classifyDesktopFile("screenshot.PNG")).toBe("Images");
    expect(classifyDesktopFile("mockup.webp")).toBe("Images");
  });

  it("classifies documents", () => {
    expect(classifyDesktopFile("proposal.pdf")).toBe("Docs");
    expect(classifyDesktopFile("notes.md")).toBe("Docs");
  });

  it("classifies archives, installers, videos, and project files", () => {
    expect(classifyDesktopFile("backup.7z")).toBe("Archives");
    expect(classifyDesktopFile("setup.msi")).toBe("Installers");
    expect(classifyDesktopFile("demo.mov")).toBe("Videos");
    expect(classifyDesktopFile("app.tsx")).toBe("Projects");
  });

  it("puts unknown and extensionless files into inbox", () => {
    expect(classifyDesktopFile("untitled")).toBe("Inbox");
    expect(classifyDesktopFile("data.unknown")).toBe("Inbox");
  });
});

describe("buildOrganizerPlan", () => {
  it("groups scanned desktop files by category without moving anything", () => {
    const plan = buildOrganizerPlan([
      { name: "screen.png", path: "C:\\Users\\NEX\\Desktop\\screen.png", category: "Images", label: "图片" },
      { name: "proposal.pdf", path: "C:\\Users\\NEX\\Desktop\\proposal.pdf", category: "Docs", label: "文档" },
      { name: "clip.mov", path: "C:\\Users\\NEX\\Desktop\\clip.mov", category: "Videos", label: "视频" },
    ]);

    expect(plan.rootFolderName).toBe("Lumora整理");
    expect(plan.totalFiles).toBe(3);
    expect(plan.groups.map((group) => [group.category, group.label, group.count])).toEqual([
      ["Images", "图片", 1],
      ["Docs", "文档", 1],
      ["Videos", "视频", 1],
    ]);
  });
});
