import type { ReviewPaneId } from "./review-panes/types";

export const layoutPreferenceStorageKey = "quireforge-workbench-layout";
export const layoutPreferenceSchemaVersion = 1;
export const reviewPaneWidthMinimum = 360;
export const reviewPaneWidthMaximum = 560;
export const terminalDockHeightMinimum = 220;
export const terminalDockHeightMaximum = 560;

export interface WorkbenchLayoutPreferences {
  schemaVersion: 1;
  reviewPaneWidth: number;
  terminalDockHeight: number;
  selectedReviewPane: ReviewPaneId;
}

export const defaultWorkbenchLayoutPreferences: WorkbenchLayoutPreferences = {
  schemaVersion: layoutPreferenceSchemaVersion,
  reviewPaneWidth: 480,
  terminalDockHeight: 320,
  selectedReviewPane: "files",
};

const reviewPaneIds = new Set<ReviewPaneId>([
  "files",
  "diff",
  "git",
  "preview",
  "activity",
  "approval",
  "review",
]);

function validDimension(
  value: unknown,
  minimum: number,
  maximum: number,
): value is number {
  return (
    typeof value === "number" &&
    Number.isInteger(value) &&
    value >= minimum &&
    value <= maximum
  );
}

export function restoreWorkbenchLayoutPreferences(
  raw: string | null,
): WorkbenchLayoutPreferences {
  if (!raw || raw.length > 512) return defaultWorkbenchLayoutPreferences;
  try {
    const parsed: unknown = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return defaultWorkbenchLayoutPreferences;
    }
    const record = parsed as Record<string, unknown>;
    if (
      Object.keys(record).length !== 4 ||
      record.schemaVersion !== layoutPreferenceSchemaVersion ||
      !validDimension(
        record.reviewPaneWidth,
        reviewPaneWidthMinimum,
        reviewPaneWidthMaximum,
      ) ||
      !validDimension(
        record.terminalDockHeight,
        terminalDockHeightMinimum,
        terminalDockHeightMaximum,
      ) ||
      typeof record.selectedReviewPane !== "string" ||
      !reviewPaneIds.has(record.selectedReviewPane as ReviewPaneId)
    ) {
      return defaultWorkbenchLayoutPreferences;
    }
    return {
      schemaVersion: layoutPreferenceSchemaVersion,
      reviewPaneWidth: record.reviewPaneWidth,
      terminalDockHeight: record.terminalDockHeight,
      selectedReviewPane: record.selectedReviewPane as ReviewPaneId,
    };
  } catch {
    return defaultWorkbenchLayoutPreferences;
  }
}

export function clampLayoutDimension(
  value: number,
  minimum: number,
  maximum: number,
) {
  return Math.min(maximum, Math.max(minimum, Math.round(value)));
}
