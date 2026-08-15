import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { WorkAdapterCatalog } from "./WorkAdapterCatalog";
import { mockInferenceCatalogSchema } from "./lib/mockInference";

describe("WorkAdapterCatalog", () => {
  it("offers deterministic local descriptors without a connection action", async () => {
    render(
      <WorkAdapterCatalog
        loadCatalog={() =>
          Promise.resolve(
            mockInferenceCatalogSchema.parse({
              schemaVersion: 1,
              profiles: [
                {
                  id: "fixture",
                  providerLabel: "Fictional local",
                  endpointLabel: "Local fixture endpoint",
                  modelLabel: "Fixture model",
                  adapterLabel: "Fixture adapter",
                  scenario: "structured",
                  descriptorSha256: "a".repeat(64),
                  capabilityProfileSha256: "b".repeat(64),
                },
              ],
            }),
          )
        }
      />,
    );
    await waitFor(() =>
      expect(
        screen.getByRole("radio", { name: /fixture adapter/i }),
      ).toBeChecked(),
    );
    expect(screen.getByText(/does not connect, transmit/i)).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /connect|submit/i }),
    ).not.toBeInTheDocument();
  });
});
