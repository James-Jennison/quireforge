import { describe, expect, it } from "vitest";
import { browserResearchPrepareRequestSchema } from "./browserResearch";

describe("browser research contracts", () => {
  it("permits only the explicitly approved exact Google scope", () => {
    expect(
      browserResearchPrepareRequestSchema.parse({
        projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
        taskId: null,
        target: "https://google.com/",
        origin: "https://google.com",
        observationLimit: 512,
      }).origin,
    ).toBe("https://google.com");
    expect(() =>
      browserResearchPrepareRequestSchema.parse({
        projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
        taskId: null,
        target: "https://www.google.com/",
        origin: "https://www.google.com",
        observationLimit: 512,
      }),
    ).toThrow();
  });
});
