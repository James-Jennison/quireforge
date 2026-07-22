import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { IntegrationCenter } from "./IntegrationCenter";
import {
  integrationCatalogSchema,
  integrationControlResultSchema,
  integrationMutationPreviewSchema,
  integrationMutationResultSchema,
  scaffoldIntegrationCatalog,
  scaffoldIntegrationControlPreview,
  scaffoldIntegrationControlResult,
  scaffoldIntegrationMutationPreview,
} from "./lib/integration";

const readyCatalog = integrationCatalogSchema.parse({
  ...scaffoldIntegrationCatalog,
  capabilities: scaffoldIntegrationCatalog.capabilities.map((capability) =>
    ["plugin.install", "plugin.remove", "marketplace.configure"].includes(
      capability.id,
    )
      ? {
          ...capability,
          availability: "ready",
          implementation: "ready",
          diagnosticCode: null,
        }
      : capability,
  ),
});

const controlReadyCatalog = integrationCatalogSchema.parse({
  ...readyCatalog,
  capabilities: readyCatalog.capabilities.map((capability) =>
    ["connector.authorize", "skill.configure", "mcp.authorize"].includes(
      capability.id,
    )
      ? {
          ...capability,
          availability: "ready",
          implementation: "ready",
          diagnosticCode: null,
        }
      : capability,
  ),
});

function renderCenter(
  overrides: Partial<React.ComponentProps<typeof IntegrationCenter>> = {},
) {
  const props: React.ComponentProps<typeof IntegrationCenter> = {
    availability: "native",
    snapshot: readyCatalog,
    preview: null,
    result: null,
    controlPreview: null,
    controlResult: null,
    busy: false,
    actionError: false,
    onRefresh: vi.fn().mockResolvedValue(undefined),
    onPreview: vi.fn().mockResolvedValue(undefined),
    onConfirm: vi.fn().mockResolvedValue(undefined),
    onControlPreview: vi.fn().mockResolvedValue(undefined),
    onControlConfirm: vi.fn().mockResolvedValue(undefined),
    onControlOpen: vi.fn().mockResolvedValue(undefined),
    onControlPoll: vi.fn().mockResolvedValue(undefined),
    onCancel: vi.fn(),
    ...overrides,
  };
  const view = render(<IntegrationCenter {...props} />);
  return Object.assign(props, { unmount: view.unmount });
}

describe("IntegrationCenter", () => {
  it("searches and filters category-preserving normalized entries", () => {
    renderCenter();

    expect(screen.getByText("5 of 5 integrations")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Category"), {
      target: { value: "plugin" },
    });
    expect(screen.getByText("1 of 5 integrations")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Fixture review plugin" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/requires separate hook trust/u),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("Fixture calendar connector"),
    ).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Search integrations"), {
      target: { value: "missing integration" },
    });
    expect(
      screen.getByText("No integrations match these filters."),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Clear filters" }));
    expect(screen.getByText("5 of 5 integrations")).toBeInTheDocument();
  });

  it("requests only fixed entry operations and a validated pinned marketplace", async () => {
    const props = renderCenter();

    fireEvent.change(screen.getByLabelText("Category"), {
      target: { value: "plugin" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Install plugin" }));
    await waitFor(() =>
      expect(props.onPreview).toHaveBeenCalledWith({
        operation: "plugin-install",
        targetEntryId: "plugin:fixture-review",
        repository: null,
        reference: null,
      }),
    );

    const marketplaceButton = screen.getByRole("button", {
      name: "Review marketplace",
    });
    expect(marketplaceButton).toBeDisabled();
    fireEvent.change(screen.getByLabelText("Repository"), {
      target: { value: "fixture/catalog" },
    });
    fireEvent.change(screen.getByLabelText("Pinned reference"), {
      target: { value: "a".repeat(40) },
    });
    expect(marketplaceButton).toBeEnabled();
    fireEvent.click(marketplaceButton);
    await waitFor(() =>
      expect(props.onPreview).toHaveBeenLastCalledWith({
        operation: "marketplace-add",
        targetEntryId: null,
        repository: "fixture/catalog",
        reference: "a".repeat(40),
      }),
    );
  });

  it("reviews permissions and warnings before consuming confirmation", async () => {
    const props = renderCenter({ preview: scaffoldIntegrationMutationPreview });

    expect(screen.getByRole("dialog")).toHaveAccessibleName("Install plugin");
    expect(
      screen.getByText("This operation uses a repository source."),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Authentication, if needed, remains a separate action."),
    ).toBeInTheDocument();
    const cancel = screen.getByRole("button", { name: "Cancel" });
    const confirm = screen.getByRole("button", { name: "Confirm change" });
    expect(cancel).toHaveFocus();
    confirm.focus();
    fireEvent.keyDown(confirm, { key: "Tab" });
    expect(cancel).toHaveFocus();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(props.onCancel).toHaveBeenCalledOnce();
    fireEvent.click(confirm);
    await waitFor(() =>
      expect(props.onConfirm).toHaveBeenCalledWith(
        scaffoldIntegrationMutationPreview.confirmationId,
      ),
    );
  });

  it("reviews and advances an official MCP authorization handoff", async () => {
    const props = renderCenter({
      snapshot: controlReadyCatalog,
      controlPreview: scaffoldIntegrationControlPreview,
    });

    expect(screen.getByRole("dialog")).toHaveAccessibleName(
      "Authorize MCP server",
    );
    expect(
      screen.getByText(/exact authorization URL returned by Codex/u),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Confirm action" }));
    await waitFor(() =>
      expect(props.onControlConfirm).toHaveBeenCalledWith(
        scaffoldIntegrationControlPreview.confirmationId,
      ),
    );
  });

  it("opens and polls only the opaque authorization action", async () => {
    const props = renderCenter({
      snapshot: controlReadyCatalog,
      controlResult: scaffoldIntegrationControlResult,
    });
    expect(
      screen.getByRole("button", { name: "Refresh catalog" }),
    ).toBeDisabled();
    fireEvent.change(screen.getByLabelText("Category"), {
      target: { value: "plugin" },
    });
    expect(
      screen.getByRole("button", { name: "Install plugin" }),
    ).toBeDisabled();
    fireEvent.click(
      screen.getByRole("button", { name: "Open authorization in browser" }),
    );
    await waitFor(() =>
      expect(props.onControlOpen).toHaveBeenCalledWith(
        scaffoldIntegrationControlResult.actionId,
      ),
    );
    props.unmount();

    const pending = integrationControlResultSchema.parse({
      ...scaffoldIntegrationControlResult,
      state: "pending",
      browserHandoffAvailable: false,
    });
    const secondProps = {
      ...props,
      controlResult: pending,
    };
    render(
      <IntegrationCenter
        {...secondProps}
        controlPreview={null}
        controlResult={pending}
      />,
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Check authorization" }),
    );
    await waitFor(() =>
      expect(props.onControlPoll).toHaveBeenCalledWith(pending.actionId),
    );
  });

  it("requests only catalog-backed integration controls", async () => {
    const props = renderCenter({ snapshot: controlReadyCatalog });
    fireEvent.change(screen.getByLabelText("Category"), {
      target: { value: "mcp-server" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Authorize MCP server" }),
    );
    await waitFor(() =>
      expect(props.onControlPreview).toHaveBeenCalledWith({
        operation: "mcp-authorize",
        targetEntryId: "mcp:fixture-knowledge",
      }),
    );
  });

  it("keeps unsupported management unavailable and reports bounded failures", () => {
    const unavailable = integrationMutationResultSchema.parse({
      schemaVersion: 1,
      state: "unavailable",
      operation: null,
      targetEntryId: null,
      catalogRefreshRequired: false,
      diagnosticCode: "stale-preview",
    });
    renderCenter({ actionError: true, result: unavailable });

    fireEvent.change(screen.getByLabelText("Category"), {
      target: { value: "mcp-server" },
    });
    expect(
      screen.getByText(/Unsupported configuration and repair paths/u),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/No raw error or integration configuration/u),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/not applied \(stale-preview\)/u),
    ).toBeInTheDocument();
  });

  it("closes a blocked preview without presenting a confirmation action", () => {
    const blocked = integrationMutationPreviewSchema.parse({
      schemaVersion: 1,
      state: "blocked",
      operation: "marketplace-remove",
      targetEntryId: "marketplace:fixture-project",
      targetDisplayName: null,
      source: "unknown",
      permissions: [],
      warnings: [],
      destructive: true,
      confirmationId: null,
      diagnosticCode: "policy-blocked",
    });
    const props = renderCenter({ preview: blocked });

    expect(screen.getByText(/operation is blocked/u)).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Confirm/u }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(props.onCancel).toHaveBeenCalledOnce();
  });
});
