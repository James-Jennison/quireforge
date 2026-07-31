import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { TaskTemplateWorkbench } from "./TaskTemplateWorkbench";

const builtinId = "01980a10-0000-7000-8000-000000000001";
const localId = "01980a10-0000-7000-8000-000000000002";
const handle = "01980a10-0000-7000-8000-000000000003";
const digest = "a".repeat(64);
const catalog = {
  schemaVersion: 1 as const,
  state: "ready" as const,
  templates: [
    {
      id: builtinId,
      title: "Built-in planning",
      purpose: "Bounded plan",
      origin: "built-in" as const,
      state: "active" as const,
    },
    {
      id: localId,
      title: "Local planning",
      purpose: "Local plan",
      origin: "local" as const,
      state: "active" as const,
    },
  ],
  capacity: {
    recordCount: 2,
    canonicalBytes: 100,
    warning: false,
    countLimit: 64 as const,
    canonicalByteLimit: 2_097_152,
  },
  diagnosticCode: null,
};
const detail = (
  id = localId,
  origin: "built-in" | "local" = "local",
  state: "active" | "archived" = "active",
) => ({
  schemaVersion: 1 as const,
  state: "ready" as const,
  template: {
    id,
    title: origin === "built-in" ? "Built-in planning" : "Local planning",
    purpose: origin === "built-in" ? "Bounded plan" : "Local plan",
    instructions: "Every instruction is fully visible.",
    origin,
    state,
    version: 7,
    sha256: digest,
  },
  mutationHandle: handle,
  diagnosticCode: null,
});
function operations(overrides = {}) {
  return {
    loadCatalog: vi.fn().mockResolvedValue(catalog),
    inspect: vi.fn(({ templateId }: { templateId: string }) =>
      Promise.resolve(
        detail(templateId, templateId === builtinId ? "built-in" : "local"),
      ),
    ),
    create: vi.fn().mockResolvedValue(detail()),
    edit: vi.fn().mockResolvedValue(detail()),
    duplicate: vi.fn().mockResolvedValue(detail()),
    archive: vi.fn().mockResolvedValue(detail(localId, "local", "archived")),
    restore: vi.fn().mockResolvedValue(detail()),
    delete: vi.fn().mockResolvedValue({
      schemaVersion: 1,
      state: "ready",
      applied: false,
      cancelled: false,
      diagnosticCode: null,
    }),
    ...overrides,
  };
}

describe("task template management workbench", () => {
  it("renders empty and closed unavailable catalog states", async () => {
    const empty = operations({
      loadCatalog: vi.fn().mockResolvedValue({ ...catalog, templates: [] }),
    });
    const { rerender } = render(
      <TaskTemplateWorkbench onClose={vi.fn()} operations={empty} />,
    );
    await screen.findByText("No task templates are available.");
    const unavailable = operations({
      loadCatalog: vi.fn().mockResolvedValue({
        schemaVersion: 1,
        state: "unavailable",
        templates: [],
        capacity: null,
        diagnosticCode: "metadata-unavailable",
      }),
    });
    rerender(
      <TaskTemplateWorkbench onClose={vi.fn()} operations={unavailable} />,
    );
    await screen.findByRole("alert");
  });

  it("loads lazily with native catalog states and renders full inspectable details", async () => {
    const api = operations();
    render(<TaskTemplateWorkbench onClose={vi.fn()} operations={api} />);
    expect(screen.getByText("Loading task templates…")).toHaveTextContent(
      "Loading task templates",
    );
    await screen.findByText("Built-in planning");
    fireEvent.click(screen.getByRole("button", { name: /built-in planning/i }));
    await screen.findByText("Every instruction is fully visible.");
    expect(screen.getByText(digest)).toBeInTheDocument();
    expect(screen.getByText("7")).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Edit" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Duplicate" }),
    ).toBeInTheDocument();
  });

  it("uses only lifecycle operations, keyboard list navigation, and native capacity warnings", async () => {
    const api = operations({
      loadCatalog: vi.fn().mockResolvedValue({
        ...catalog,
        capacity: { ...catalog.capacity, warning: true },
      }),
    });
    render(<TaskTemplateWorkbench onClose={vi.fn()} operations={api} />);
    await screen.findByRole("alert");
    const list = screen.getByRole("listbox", { name: "Task templates" });
    fireEvent.keyDown(list, { key: "End" });
    await waitFor(() =>
      expect(api.inspect).toHaveBeenCalledWith({ templateId: localId }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Archive" }));
    await waitFor(() =>
      expect(api.archive).toHaveBeenCalledWith({ mutationHandle: handle }),
    );
    expect(api).not.toHaveProperty("preview");
    expect(api).not.toHaveProperty("confirm");
  });

  it("preserves authored drafts on native failure and refreshes after successful lifecycle work", async () => {
    const api = operations({
      create: vi.fn().mockResolvedValue({
        schemaVersion: 1,
        state: "unavailable",
        template: null,
        mutationHandle: null,
        diagnosticCode: "capacity-reached",
      }),
    });
    render(<TaskTemplateWorkbench onClose={vi.fn()} operations={api} />);
    await screen.findByText("Local planning");
    fireEvent.click(screen.getByRole("button", { name: "New local template" }));
    fireEvent.change(screen.getByRole("textbox", { name: /title/i }), {
      target: { value: "Preserved draft" },
    });
    fireEvent.change(screen.getByRole("textbox", { name: /purpose/i }), {
      target: { value: "Purpose" },
    });
    fireEvent.change(screen.getByRole("textbox", { name: /instructions/i }), {
      target: { value: "Instructions" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Save local template" }),
    );
    await screen.findByText(/capacity has been reached/i);
    expect(screen.getByRole("textbox", { name: /title/i })).toHaveValue(
      "Preserved draft",
    );
    expect(api.loadCatalog).toHaveBeenCalledTimes(1);
  });

  it("requires destructive confirmation with Cancel focused and restores focus", async () => {
    const api = operations({
      inspect: vi.fn().mockResolvedValue(detail(localId, "local", "archived")),
    });
    render(<TaskTemplateWorkbench onClose={vi.fn()} operations={api} />);
    await screen.findByText("Local planning");
    fireEvent.click(screen.getByRole("button", { name: /local planning/i }));
    await screen.findByRole("button", { name: "Delete permanently" });
    const trigger = screen.getByRole("button", { name: "Delete permanently" });
    fireEvent.click(trigger);
    const dialog = screen.getByRole("dialog", {
      name: /delete local template/i,
    });
    expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus();
    fireEvent.keyDown(dialog, { key: "Escape" });
    await waitFor(() => expect(trigger).toHaveFocus());
    expect(api.delete).not.toHaveBeenCalled();
  });
});
