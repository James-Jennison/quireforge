import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ContextAssemblyWorkbench } from "./ContextAssemblyWorkbench";

describe("ContextAssemblyWorkbench", () => {
  it("labels the local-only authority boundary and starts with no selected source", () => {
    render(
      <ContextAssemblyWorkbench
        projectId="019fbee6-476f-71b0-853c-f067657aa69c"
        projectLabel="Release review"
        onClose={() => undefined}
      />,
    );
    expect(
      screen.getByRole("heading", { name: /governed context review/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/nothing is selected by default/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Project scope: Release review/i),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("019fbee6-476f-71b0-853c-f067657aa69c"),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /prepare review/i }),
    ).toBeDisabled();
  });
});
