import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ContextAssemblyWorkbench } from "./ContextAssemblyWorkbench";

describe("ContextAssemblyWorkbench", () => {
  it("labels the local-only authority boundary and starts with no selected source", () => {
    render(
      <ContextAssemblyWorkbench projectId={null} onClose={() => undefined} />,
    );
    expect(
      screen.getByRole("heading", { name: /governed context review/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/nothing is selected by default/i),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /prepare review/i }),
    ).toBeDisabled();
  });
});
