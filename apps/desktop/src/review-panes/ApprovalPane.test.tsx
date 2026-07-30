import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import ApprovalPane from "./ApprovalPane";
import {
  resetLocalReviewSessionForTest,
  setLocalReviewPromotionPresentation,
} from "./localReviewSession";
import type { ReviewPaneData } from "./types";

const props = {
  conversation: { pendingApproval: null },
} as unknown as ReviewPaneData;
afterEach(resetLocalReviewSessionForTest);

describe("local review Approval presentation", () => {
  it("shows prepared promotion metadata read-only and separates approval", () => {
    setLocalReviewPromotionPresentation({
      state: "prepared",
      label: "Review text",
      destinationClass: "text",
      sha256: "a".repeat(64),
      expiresAtMs: 300000,
    });
    render(<ApprovalPane {...props} />);
    expect(
      screen.getByText("Review and promotion do not approve or dispatch work."),
    ).toBeVisible();
    expect(screen.getByText("prepared")).toBeVisible();
    expect(screen.getByText("Destination: text")).toBeVisible();
    expect(
      screen.queryByRole("button", {
        name: /prepare|create transient|confirm|cancel|save|run|dispatch|publish|deploy/i,
      }),
    ).toBeNull();
  });
  it("renders succeeded and expired status without changing approval", () => {
    setLocalReviewPromotionPresentation({
      state: "succeeded",
      label: "Review text",
    });
    const { rerender } = render(<ApprovalPane {...props} />);
    expect(screen.getByText("succeeded")).toBeVisible();
    setLocalReviewPromotionPresentation({ state: "expired" });
    rerender(<ApprovalPane {...props} />);
    expect(screen.getByText("expired")).toBeVisible();
  });
});
