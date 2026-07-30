import { useSyncExternalStore } from "react";

export const localReviewActivityKinds = [
  "collection-created",
  "collection-resumed",
  "collection-discarded",
  "item-added",
  "item-discarded",
  "annotation-added",
  "annotation-edited",
  "annotation-resolved",
  "annotation-reopened",
  "annotation-deleted",
  "comparison-created",
  "comparison-discarded",
  "promotion-prepared",
  "promotion-canceled",
  "promotion-expired",
  "promotion-succeeded",
  "promotion-failed",
] as const;
export type LocalReviewActivityKind = (typeof localReviewActivityKinds)[number];
export interface LocalReviewActivityEvent {
  id: string;
  kind: LocalReviewActivityKind;
  label: string;
  timestamp: number;
  status: "success" | "failed" | "info";
  digest?: string;
  reason?: string;
}
export interface LocalReviewPromotionPresentation {
  state: "eligible" | "prepared" | "expired" | "succeeded" | "unavailable";
  label?: string;
  destinationClass?: string;
  sha256?: string;
  expiresAtMs?: number;
}

let events: LocalReviewActivityEvent[] = [];
let promotion: LocalReviewPromotionPresentation = { state: "unavailable" };
const listeners = new Set<() => void>();
const notify = () => listeners.forEach((listener) => listener());
const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => listeners.delete(listener);
};
export const recordLocalReviewActivity = (
  event: Omit<LocalReviewActivityEvent, "id" | "timestamp">,
) => {
  events = [
    { ...event, id: crypto.randomUUID(), timestamp: Date.now() },
    ...events,
  ].slice(0, 12);
  notify();
};
export const setLocalReviewPromotionPresentation = (
  next: LocalReviewPromotionPresentation,
) => {
  promotion = next;
  notify();
};
export const useLocalReviewActivity = () =>
  useSyncExternalStore(
    subscribe,
    () => events,
    () => [],
  );
export const useLocalReviewPromotionPresentation = () =>
  useSyncExternalStore(
    subscribe,
    () => promotion,
    (): LocalReviewPromotionPresentation => ({ state: "unavailable" }),
  );
export const resetLocalReviewSessionForTest = () => {
  events = [];
  promotion = { state: "unavailable" };
  notify();
};
