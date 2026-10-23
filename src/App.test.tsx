import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
const { setTitleMock } = vi.hoisted(() => ({ setTitleMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle: setTitleMock }),
}));
vi.mock("sonner", () => ({
  toast: Object.assign(vi.fn(), {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  }),
  Toaster: () => null,
}));

import App from "./App";

describe("App 外壳", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    setTitleMock.mockReset();
    invokeMock.mockImplementation(async (cmd: string) => {
      switch (cmd) {
        case "scan_input_methods":
          return {
            candidates: [],
            warningCode: null,
            warningArguments: null,
          };
        case "get_language_preference":
          return "zh-Hans";
        case "get_app_version":
          return "2026.10.20";
        case "list_custom_game_paths":
        case "list_custom_input_methods":
          return [];
        default:
          return undefined;
      }
    });
  });

  it("侧边栏两项都带图标", async () => {
    render(<App />);

    const home = await screen.findByRole("button", { name: "主页" });
    const settings = screen.getByRole("button", { name: "设置" });

    // lucide 图标是 svg，且对读屏隐藏（文案已由按钮本身提供）
    expect(home.querySelector("svg")).not.toBeNull();
    expect(settings.querySelector("svg")).not.toBeNull();
    expect(home.querySelector("svg")?.getAttribute("aria-hidden")).toBe("true");
  });

  it("点击设置切到设置页", async () => {
    const user = userEvent.setup();
    render(<App />);

    // 设置页常驻在 DOM 里但不可见，切过去才显示
    expect(screen.getByText("项目官网")).not.toBeVisible();

    await user.click(await screen.findByRole("button", { name: "设置" }));

    await waitFor(() => {
      expect(screen.getByText("项目官网")).toBeVisible();
    });
  });

  it("切到设置再切回，向导仍停在原来的步骤", async () => {
    const user = userEvent.setup();
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("第 1 步，共 3 步")).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: "下一步" }));
    await waitFor(() => {
      expect(screen.getByText("第 2 步，共 3 步")).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: "设置" }));
    await waitFor(() => {
      expect(screen.getByText("项目官网")).toBeVisible();
    });

    await user.click(screen.getByRole("button", { name: "主页" }));
    await waitFor(() => {
      expect(screen.getByText("第 2 步，共 3 步")).toBeInTheDocument();
    });
  });

  it("原生窗口标题取自 AppDisplayName", async () => {
    render(<App />);

    await waitFor(() => {
      expect(setTitleMock).toHaveBeenCalledWith("WOWS输入法工具");
    });
  });
});
