import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import ActivityPane from "./ActivityPane";
import {
  recordLocalReviewActivity,
  resetLocalReviewSessionForTest,
} from "./localReviewSession";
import type { ReviewPaneData } from "./types";

const props = { conversationEvents: [] } as unknown as ReviewPaneData;
afterEach(resetLocalReviewSessionForTest);

describe("local review Activity presentation", () => {
  it("renders bounded newest-first local review events without payloads", () => {
    for (let index = 0; index < 13; index += 1)
      recordLocalReviewActivity({
        kind: "item-added",
        label: `Item ${index}`,
        status: "success",
        digest: "a".repeat(64),
      });
    render(<ActivityPane {...props} />);
    expect(screen.getAllByText(/Local review · item-added/)).toHaveLength(12);
    expect(screen.getByText(/Item 12/)).toBeVisible();
    expect(screen.queryByText(/Item 0/)).toBeNull();
    expect(
      screen.queryByText(
        /path|url|command output|credential|provider|connector/i,
      ),
    ).toBeNull();
    expect(
      screen.getAllByLabelText(`SHA-256 ${"a".repeat(64)}`)[0],
    ).toHaveTextContent("a".repeat(12));
  });
  it("has an empty state for a new current session", () => {
    render(<ActivityPane {...props} />);
    expect(
      screen.getByText("No bounded task activity is available."),
    ).toBeVisible();
  });
});
