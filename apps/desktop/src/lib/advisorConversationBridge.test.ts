import { describe, expect, it, vi } from "vitest";

import { scaffoldAdvisorConversation } from "./advisorConversation";
import {
  ADVISOR_CONVERSATION_POLL_COMMAND,
  ADVISOR_CONVERSATION_START_COMMAND,
  ADVISOR_CONVERSATION_STATUS_COMMAND,
  loadAdvisorConversation,
  pollAdvisorConversation,
  startAdvisorConversation,
} from "./bridge";

describe("Advisor conversation bridge", () => {
  it("uses only typed bounded Advisor commands", async () => {
    const invoke = vi.fn().mockResolvedValue(scaffoldAdvisorConversation);
    await expect(loadAdvisorConversation(invoke)).resolves.toEqual(
      scaffoldAdvisorConversation,
    );
    await expect(
      startAdvisorConversation(
        {
          prompt: "Review this safe summary.",
          projectId: "018f0000-0000-7000-8000-000000000001",
          attachmentId: null,
          attachmentManifestSha256: null,
          attachmentConfirmation: null,
          imageAttachmentId: null,
          imageAttachmentManifestSha256: null,
          imageAttachmentConfirmation: null,
          documentAttachmentId: null,
          documentAttachmentManifestSha256: null,
          documentAttachmentConfirmation: null,
          archiveAttachmentId: null,
          archiveAttachmentManifestSha256: null,
          archiveAttachmentConfirmation: null,
        },
        invoke,
      ),
    ).resolves.toEqual(scaffoldAdvisorConversation);
    await expect(
      pollAdvisorConversation("018f0000-0000-7000-8000-000000000001", invoke),
    ).resolves.toEqual(scaffoldAdvisorConversation);

    expect(invoke).toHaveBeenNthCalledWith(
      1,
      ADVISOR_CONVERSATION_STATUS_COMMAND,
    );
    expect(invoke).toHaveBeenNthCalledWith(
      2,
      ADVISOR_CONVERSATION_START_COMMAND,
      {
        request: {
          prompt: "Review this safe summary.",
          projectId: "018f0000-0000-7000-8000-000000000001",
          attachmentId: null,
          attachmentManifestSha256: null,
          attachmentConfirmation: null,
          imageAttachmentId: null,
          imageAttachmentManifestSha256: null,
          imageAttachmentConfirmation: null,
          documentAttachmentId: null,
          documentAttachmentManifestSha256: null,
          documentAttachmentConfirmation: null,
          archiveAttachmentId: null,
          archiveAttachmentManifestSha256: null,
          archiveAttachmentConfirmation: null,
        },
      },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      3,
      ADVISOR_CONVERSATION_POLL_COMMAND,
      {
        conversationId: "018f0000-0000-7000-8000-000000000001",
      },
    );
  });

  it("rejects path-shaped context input before native IPC", async () => {
    const invoke = vi.fn();
    await expect(
      startAdvisorConversation(
        {
          prompt: "Review this",
          projectId: "/tmp/unsafe",
          attachmentId: null,
          attachmentManifestSha256: null,
          attachmentConfirmation: null,
          imageAttachmentId: null,
          imageAttachmentManifestSha256: null,
          imageAttachmentConfirmation: null,
          documentAttachmentId: null,
          documentAttachmentManifestSha256: null,
          documentAttachmentConfirmation: null,
          archiveAttachmentId: null,
          archiveAttachmentManifestSha256: null,
          archiveAttachmentConfirmation: null,
        },
        invoke,
      ),
    ).rejects.toThrow();
    expect(invoke).not.toHaveBeenCalled();
  });
});
