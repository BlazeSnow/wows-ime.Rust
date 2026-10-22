import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));
const { openMock } = vi.hoisted(() => ({ openMock: vi.fn() }));
const { openPathMock } = vi.hoisted(() => ({ openPathMock: vi.fn() }));
const { toastSuccess, toastError, toastWarning, toastInfo } = vi.hoisted(() => ({
  toastSuccess: vi.fn(),
  toastError: vi.fn(),
  toastWarning: vi.fn(),
  toastInfo: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: openMock }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openPath: openPathMock }));
vi.mock("sonner", () => ({
  toast: Object.assign(vi.fn(), {
    success: toastSuccess,
    error: toastError,
    warning: toastWarning,
    info: toastInfo,
  }),
  Toaster: () => null,
}));

import { HomePage } from "./HomePage";

const SCANNED = {
  candidates: [
    { displayName: "微软拼音", category: "ChineseSimplified", confidence: 10 },
    { displayName: "微軟速成", category: "ChineseTraditional", confidence: 10 },
  ],
  warningCode: null,
  warningArguments: null,
};

type Overrides = Record<string, unknown>;

function stubInvoke(overrides: Overrides = {}) {
  invokeMock.mockImplementation(async (cmd: string, args?: Record<string, unknown>) => {
    if (cmd in overrides) {
      const value = overrides[cmd];
      return typeof value === "function"
        ? (value as (a?: Record<string, unknown>) => unknown)(args)
        : value;
    }
    switch (cmd) {
      case "scan_input_methods":
        return SCANNED;
      case "list_custom_game_paths":
        return [];
      case "list_custom_input_methods":
        return [];
      case "get_selected_game_path":
        return "";
      case "has_game_executable":
        return true;
      case "resolve_target_config_files":
        return ["C:\\Games\\bin\\1\\res_mods\\ime_config.xml"];
      case "existing_config_count":
        return 0;
      case "write_config_files":
        return 1;
      default:
        return undefined;
    }
  });
}

async function renderHome() {
  render(<HomePage />);
  // 首屏要等游戏路径与扫描结果加载完
  await waitFor(() => {
    expect(screen.getByText("第 1 步，共 3 步")).toBeInTheDocument();
  });
}

describe("HomePage 三步向导", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    openMock.mockReset();
    openPathMock.mockReset();
    toastSuccess.mockReset();
    toastError.mockReset();
    toastWarning.mockReset();
    toastInfo.mockReset();
    stubInvoke();
  });

  it("首步列出三个预设游戏路径并默认选中第一个", async () => {
    await renderHome();

    expect(screen.getByText("Steam（国际服）")).toBeInTheDocument();
    expect(screen.getByText("Lesta（俄服）")).toBeInTheDocument();
    expect(screen.getByText("360（国服）")).toBeInTheDocument();

    const radios = screen.getAllByRole("radio") as HTMLInputElement[];
    expect(radios).toHaveLength(3);
    expect(radios[0].checked).toBe(true);
  });

  it("选择其它路径后立即持久化", async () => {
    const user = userEvent.setup();
    await renderHome();

    const radios = screen.getAllByRole("radio") as HTMLInputElement[];
    await user.click(radios[1]);

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("set_selected_game_path", {
        path: "C:\\Games\\Korabli",
      });
    });
  });

  it("第二步列出扫描到的输入法", async () => {
    const user = userEvent.setup();
    await renderHome();

    await user.click(screen.getByRole("button", { name: "下一步" }));

    await waitFor(() => {
      expect(screen.getByText("第 2 步，共 3 步")).toBeInTheDocument();
    });
    expect(screen.getByText("检测到的输入法")).toBeInTheDocument();
    expect(screen.getByText("微软拼音")).toBeInTheDocument();
    expect(screen.getByText("微軟速成")).toBeInTheDocument();
  });

  it("未勾选输入法时不允许进入下一步并给出提示", async () => {
    const user = userEvent.setup();
    await renderHome();

    await user.click(screen.getByRole("button", { name: "下一步" }));
    await waitFor(() => {
      expect(screen.getByText("第 2 步，共 3 步")).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: "下一步" }));

    expect(toastWarning).toHaveBeenCalledWith("请至少勾选一个输入法。");
    expect(screen.getByText("第 2 步，共 3 步")).toBeInTheDocument();
  });

  it("勾选输入法后第三步展示路径与已选输入法", async () => {
    const user = userEvent.setup();
    await renderHome();

    await user.click(screen.getByRole("button", { name: "下一步" }));
    await waitFor(() => {
      expect(screen.getByText("检测到的输入法")).toBeInTheDocument();
    });

    await user.click(screen.getAllByRole("checkbox")[0]);
    await user.click(screen.getByRole("button", { name: "下一步" }));

    await waitFor(() => {
      expect(screen.getByText("第 3 步，共 3 步")).toBeInTheDocument();
    });
    expect(screen.getByText("已选输入法")).toBeInTheDocument();
    expect(screen.getByText(/1\. 微软拼音/)).toBeInTheDocument();
    expect(screen.getByText("C:\\Program Files (x86)\\Steam\\steamapps\\common\\World of Warships")).toBeInTheDocument();
  });

  it("写入前弹出新增确认，确认后按选中项调用写入命令", async () => {
    const user = userEvent.setup();
    await renderHome();

    await user.click(screen.getByRole("button", { name: "下一步" }));
    await waitFor(() => {
      expect(screen.getByText("检测到的输入法")).toBeInTheDocument();
    });
    await user.click(screen.getAllByRole("checkbox")[0]);
    await user.click(screen.getByRole("button", { name: "下一步" }));

    await waitFor(() => {
      expect(screen.getByText("第 3 步，共 3 步")).toBeInTheDocument();
    });
    await user.click(screen.getByRole("button", { name: "写入配置" }));

    await waitFor(() => {
      expect(screen.getByText("未发现配置文件")).toBeInTheDocument();
    });
    expect(screen.getByText("将新增 1 个 ime_config.xml，是否继续？")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "新增" }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "write_config_files",
        expect.objectContaining({
          inputMethods: [
            { displayName: "微软拼音", category: "ChineseSimplified" },
          ],
        }),
      );
    });
    expect(toastSuccess).toHaveBeenCalledWith("写入成功，共更新 1 个配置文件。");
  });

  it("目标已有配置文件时改为覆盖确认", async () => {
    const user = userEvent.setup();
    stubInvoke({ existing_config_count: 3 });
    await renderHome();

    await user.click(screen.getByRole("button", { name: "下一步" }));
    await waitFor(() => {
      expect(screen.getByText("检测到的输入法")).toBeInTheDocument();
    });
    await user.click(screen.getAllByRole("checkbox")[0]);
    await user.click(screen.getByRole("button", { name: "下一步" }));

    await waitFor(() => {
      expect(screen.getByText("第 3 步，共 3 步")).toBeInTheDocument();
    });
    await user.click(screen.getByRole("button", { name: "写入配置" }));

    await waitFor(() => {
      expect(screen.getByText("发现已有配置文件")).toBeInTheDocument();
    });
    expect(screen.getByText("检测到 3 个已有 ime_config.xml，是否覆盖？")).toBeInTheDocument();
  });

  it("取消写入确认后不落盘", async () => {
    const user = userEvent.setup();
    await renderHome();

    await user.click(screen.getByRole("button", { name: "下一步" }));
    await waitFor(() => {
      expect(screen.getByText("检测到的输入法")).toBeInTheDocument();
    });
    await user.click(screen.getAllByRole("checkbox")[0]);
    await user.click(screen.getByRole("button", { name: "下一步" }));

    await waitFor(() => {
      expect(screen.getByText("第 3 步，共 3 步")).toBeInTheDocument();
    });
    await user.click(screen.getByRole("button", { name: "写入配置" }));

    await waitFor(() => {
      expect(screen.getByText("未发现配置文件")).toBeInTheDocument();
    });
    await user.click(screen.getByRole("button", { name: "取消" }));

    expect(invokeMock).not.toHaveBeenCalledWith(
      "write_config_files",
      expect.anything(),
    );
  });

  it("游戏目录缺少游戏 exe 时中止写入", async () => {
    const user = userEvent.setup();
    stubInvoke({ has_game_executable: false });
    await renderHome();

    await user.click(screen.getByRole("button", { name: "下一步" }));
    await waitFor(() => {
      expect(screen.getByText("检测到的输入法")).toBeInTheDocument();
    });
    await user.click(screen.getAllByRole("checkbox")[0]);
    await user.click(screen.getByRole("button", { name: "下一步" }));

    await waitFor(() => {
      expect(screen.getByText("第 3 步，共 3 步")).toBeInTheDocument();
    });
    await user.click(screen.getByRole("button", { name: "写入配置" }));

    await waitFor(() => {
      expect(toastError).toHaveBeenCalledWith(
        "所选目录未找到 WorldOfWarships.exe 或 Korabli.exe。",
      );
    });
    expect(invokeMock).not.toHaveBeenCalledWith(
      "write_config_files",
      expect.anything(),
    );
  });

  it("扫描结果为空且带警告码时提示警告内容", async () => {
    stubInvoke({
      scan_input_methods: {
        candidates: [],
        warningCode: "Tsf/GetLanguageListEmpty",
        warningArguments: null,
      },
    });
    const user = userEvent.setup();
    await renderHome();

    await user.click(screen.getByRole("button", { name: "下一步" }));
    await waitFor(() => {
      expect(screen.getByText("检测到的输入法")).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(toastWarning).toHaveBeenCalledWith(
        "扫描完成，未发现输入法。TSF错误：GetLanguageList 返回空语言列表",
      );
    });
  });

  it("添加自定义输入法后写入列表并持久化", async () => {
    const user = userEvent.setup();
    await renderHome();

    await user.click(screen.getByRole("button", { name: "下一步" }));
    await waitFor(() => {
      expect(screen.getByText("检测到的输入法")).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: "自定义输入法名称" }));
    await waitFor(() => {
      expect(screen.getByText("添加自定义输入法")).toBeInTheDocument();
    });

    await user.type(screen.getByPlaceholderText("请输入输入法名称"), "我的输入法");
    await user.click(screen.getByRole("button", { name: "添加" }));

    await waitFor(() => {
      expect(screen.getByText("我的输入法")).toBeInTheDocument();
    });
    expect(toastSuccess).toHaveBeenCalledWith("已添加自定义输入法。");
    expect(invokeMock).toHaveBeenCalledWith("save_custom_input_methods", {
      inputMethods: expect.arrayContaining([
        { displayName: "我的输入法", category: "ChineseSimplified" },
      ]),
    });
  });

  it("自定义输入法名称重复时拒绝添加", async () => {
    const user = userEvent.setup();
    await renderHome();

    await user.click(screen.getByRole("button", { name: "下一步" }));
    await waitFor(() => {
      expect(screen.getByText("检测到的输入法")).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: "自定义输入法名称" }));
    await waitFor(() => {
      expect(screen.getByText("添加自定义输入法")).toBeInTheDocument();
    });

    await user.type(screen.getByPlaceholderText("请输入输入法名称"), "微软拼音");
    await user.click(screen.getByRole("button", { name: "添加" }));

    expect(toastWarning).toHaveBeenCalledWith("该输入法名称已存在。");
    expect(invokeMock).not.toHaveBeenCalledWith(
      "save_custom_input_methods",
      expect.anything(),
    );
  });
});
