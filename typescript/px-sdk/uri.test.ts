import { describe, expect, it } from "vitest";

import { parseUri, uriIdentity } from "./src/index.js";

describe("PX URI compatibility", () => {
  it("round-trips canonical px URIs", () => {
    expect(uriIdentity("px://toystory/character/woody")).toBe("px://toystory/character/woody");
    expect(parseUri("px://toystory/character/woody").repository).toBe("toystory");
  });

  it("reads a legacy URI and serializes the canonical PX form", () => {
    expect(uriIdentity("nap://toystory/character/woody")).toBe("px://toystory/character/woody");
  });

  it("rejects unsupported URI schemes", () => {
    expect(() => parseUri("other://toystory/character/woody")).toThrow("unsupported URI scheme");
  });
});
