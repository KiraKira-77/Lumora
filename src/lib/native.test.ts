import { describe, expect, it } from "vitest";
import { searchNativeFiles } from "./native";

describe("native bridge browser fallback", () => {
  it("searches preview files by query", async () => {
    const result = await searchNativeFiles("proposal");

    expect(result.query).toBe("proposal");
    expect(result.total_matches).toBe(1);
    expect(result.files[0]).toMatchObject({
      name: "proposal.pdf",
      category: "Docs",
    });
  });
});
