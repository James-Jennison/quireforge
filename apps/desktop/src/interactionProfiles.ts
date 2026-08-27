export const interactionProfileStorageKey = "quireforge-interaction-profile";

export const interactionProfiles = [
  {
    id: "direct",
    label: "Direct",
    description: "Concise, pragmatic conversational prose.",
  },
  {
    id: "conversational",
    label: "Conversational",
    description: "Warmer, more exploratory conversational prose.",
  },
] as const;

export type InteractionProfileId = (typeof interactionProfiles)[number]["id"];

export const defaultInteractionProfile: InteractionProfileId = "direct";

const profileIds = new Set<InteractionProfileId>(
  interactionProfiles.map(({ id }) => id),
);

export function restoreInteractionProfile(
  value: string | null,
): InteractionProfileId {
  return value !== null && profileIds.has(value as InteractionProfileId)
    ? (value as InteractionProfileId)
    : defaultInteractionProfile;
}

export function storedInteractionProfile(): InteractionProfileId {
  return restoreInteractionProfile(
    window.localStorage.getItem(interactionProfileStorageKey),
  );
}
