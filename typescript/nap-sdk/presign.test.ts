import { describe, expect, it } from "vitest";

import { presignRepresentation } from "./src/index.js";

describe("presignRepresentation", () => {
  it("rejects conflicting branch and commit selectors", async () => {
    await expect(
      presignRepresentation(
        "nap://test/character/hero",
        "reference_image",
        {
          repoPath: "/tmp",
          branch: "main",
          commit: "abc",
        },
      ),
    ).rejects.toThrow("either branch or commit");
  });
});
