import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
const { openPathMock } = vi.hoisted(() => ({ openPathMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openPath: openPathMock }));
vi.mock("sonner", () => ({
  toast: Object.assign(vi.fn(), {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  }),
  Toaster: () => null,
}));

import { SettingsPage } from "./SettingsPage";
import { PROJECT_REPOSITORY, PROJECT_WEBSITE } from "../lib/links";

function stubInvoke(overrides: Record<string, unknown> = {}) {
  invokeMock.mockImplementation(async (cmd: string) => {
    if (cmd in overrides) {
      return overrides[cmd];
    }
    switch (cmd) {
      case "get_app_version":
        return "2026.10.20";
      case "get_language_preference":
        return "zh-Hans";
      default:
        return undefined;
    }
  });
}

describe("SettingsPage", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    openPathMock.mockReset();
    stubInvoke();
  });

  it("展示软件版本号", async () => {
    render(<SettingsPage />);

    await waitFor(() => {
      expect(screen.getByText("2026.10.20")).toBeInTheDocument();
    });
    expect(screen.getByText("软件版本")).toBeInTheDocument();
  });

  it("语言设置只出现一次标题，不重复", async () => {
    render(<SettingsPage />);

    await waitFor(() => {
      expect(screen.getByRole("combobox")).toBeInTheDocument();
    });
    // 卡片标题即下拉框的可访问名称，页面上不应再出现第二个「语言」
    expect(screen.getAllByText("语言")).toHaveLength(1);
    // 可访问名称仍要能取到
    expect(screen.getByLabelText("语言")).toBeInTheDocument();
  });

  it("展示项目官网与仓库入口", async () => {
    render(<SettingsPage />);

    await waitFor(() => {
      expect(screen.getByText("项目官网")).toBeInTheDocument();
    });
    expect(screen.getByText("项目仓库")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "打开官网" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "打开仓库" })).toBeInTheDocument();
  });

  it("卡片展示的地址与实际打开的地址一致", async () => {
    const user = userEvent.setup();
    openPathMock.mockResolvedValue(true);
    render(<SettingsPage />);

    await waitFor(() => {
      expect(screen.getByText("项目官网")).toBeInTheDocument();
    });

    // 展示的地址直接来自链接常量，点击打开的必须是同一个地址
    expect(screen.getByText(PROJECT_WEBSITE)).toBeInTheDocument();
    expect(screen.getByText(PROJECT_REPOSITORY)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "打开官网" }));
    await waitFor(() => {
      expect(openPathMock).toHaveBeenCalledWith(PROJECT_WEBSITE);
    });

    await user.click(screen.getByRole("button", { name: "打开仓库" }));
    await waitFor(() => {
      expect(openPathMock).toHaveBeenCalledWith(PROJECT_REPOSITORY);
    });
  });

  it("点击打开官网时调用系统默认程序", async () => {
    const user = userEvent.setup();
    openPathMock.mockResolvedValue(true);
    render(<SettingsPage />);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "打开官网" })).toBeInTheDocument();
    });
    await user.click(screen.getByRole("button", { name: "打开官网" }));

    await waitFor(() => {
      expect(openPathMock).toHaveBeenCalledWith(PROJECT_WEBSITE);
    });
  });

  it("切换界面语言后立即生效并持久化", async () => {
    const user = userEvent.setup();
    render(<SettingsPage />);

    await waitFor(() => {
      expect(screen.getByLabelText("语言")).toBeInTheDocument();
    });

    // Radix Select 走键盘驱动：Enter 展开 → 方向键从「简体中文」移到「繁體中文」→ Enter 提交
    const trigger = screen.getByRole("combobox");
    trigger.focus();
    await user.keyboard("[Enter]");
    await user.keyboard("[ArrowDown]");
    await user.keyboard("[Enter]");

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("set_setting", {
        key: "Settings.Language",
        value: "zh-Hant",
      });
    });
    // 界面文案应立即切换成繁体（原 resw 中 zh-Hant 的语言标签为「語言」）
    await waitFor(() => {
      expect(screen.getByLabelText("語言")).toBeInTheDocument();
    });
  });
});
