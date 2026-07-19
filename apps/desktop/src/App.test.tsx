import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";

import App from "./App";
import { scaffoldBootstrap } from "./lib/contract";

describe("QuireForge desktop shell", () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("renders the honest scaffold state and verifies native data", async () => {
    render(<App loadBootstrap={() => Promise.resolve(scaffoldBootstrap)} />);

    expect(
      screen.getByRole("heading", {
        name: "A quiet place for ambitious work.",
      }),
    ).toBeInTheDocument();
    expect(await screen.findByText("Native IPC verified")).toBeInTheDocument();
    expect(screen.getByText("No project attached")).toBeInTheDocument();
    expect(screen.getAllByText("planned")).toHaveLength(2);
    expect(
      screen.getByText(
        /not made, endorsed, supported, or distributed by OpenAI/u,
      ),
    ).toBeInTheDocument();
  });

  it("labels a browser-only render without simulating native success", async () => {
    render(<App loadBootstrap={() => Promise.reject(new Error("no IPC"))} />);

    expect(await screen.findByText("Browser preview")).toBeInTheDocument();
    expect(screen.queryByText("Native IPC verified")).not.toBeInTheDocument();
  });

  it("persists the explicit theme choice", () => {
    render(<App loadBootstrap={() => Promise.resolve(scaffoldBootstrap)} />);

    const button = screen.getByRole("button", { name: /theme/u });
    fireEvent.click(button);

    expect(window.localStorage.getItem("quireforge-theme")).toBe(
      document.documentElement.dataset.theme,
    );
  });
});
