import { describe, expect, it } from "vitest";

import {
  defaultInteractionProfile,
  interactionProfiles,
  restoreInteractionProfile,
} from "./interactionProfiles";

describe("interaction profiles", () => {
  it("keeps a closed, authority-free profile registry", () => {
    expect(interactionProfiles.map(({ id }) => id)).toEqual([
      "direct",
      "conversational",
    ]);
    expect(defaultInteractionProfile).toBe("direct");
    expect(restoreInteractionProfile("authority")).toBe("direct");
    expect(restoreInteractionProfile("conversational")).toBe("conversational");
  });
});
