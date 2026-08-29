import { describe, expect, it } from "vitest";

import { localChatRequestSchema } from "./localChat";

describe("local chat request", () => {
  it("defaults to Direct and accepts only the two documented interaction styles", () => {
    expect(localChatRequestSchema.parse({ message: "Hello" })).toEqual({
      message: "Hello",
      interactionProfile: "direct",
    });
    expect(
      localChatRequestSchema.parse({
        message: "Hello",
        interactionProfile: "conversational",
      }),
    ).toEqual({ message: "Hello", interactionProfile: "conversational" });
    expect(() =>
      localChatRequestSchema.parse({
        message: "Hello",
        interactionProfile: "other",
      }),
    ).toThrow();
  });
});
