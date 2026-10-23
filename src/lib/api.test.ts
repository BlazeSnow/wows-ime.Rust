import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import {
  existingConfigCount,
  getAppVersion,
  getDataDir,
  getLanguagePreference,
  getSelectedGamePath,
  getSetting,
  hasGameExecutable,
  listCustomGamePaths,
  listCustomInputMethods,
  resolveTargetConfigFiles,
  saveCustomGamePaths,
  saveCustomInputMethods,
  saveSelectedGamePath,
  scanInputMethods,
  setSetting,
  writeConfigFiles,
} from "./api";

describe("api 层命令契约", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockResolvedValue(undefined);
  });

  it("只读命令的调用名与参数固定", async () => {
    await getAppVersion();
    expect(invokeMock).toHaveBeenLastCalledWith("get_app_version");

    await getDataDir();
    expect(invokeMock).toHaveBeenLastCalledWith("get_data_dir");

    await getLanguagePreference();
    expect(invokeMock).toHaveBeenLastCalledWith("get_language_preference");

    await getSetting("Settings.Language");
    expect(invokeMock).toHaveBeenLastCalledWith("get_setting", {
      key: "Settings.Language",
    });

    await hasGameExecutable("C:\\Games");
    expect(invokeMock).toHaveBeenLastCalledWith("has_game_executable", {
      gameRoot: "C:\\Games",
    });

    await resolveTargetConfigFiles("C:\\Games");
    expect(invokeMock).toHaveBeenLastCalledWith("resolve_target_config_files", {
      gameRoot: "C:\\Games",
    });

    await existingConfigCount("C:\\Games");
    expect(invokeMock).toHaveBeenLastCalledWith("existing_config_count", {
      gameRoot: "C:\\Games",
    });

    await getSelectedGamePath();
    expect(invokeMock).toHaveBeenLastCalledWith("get_selected_game_path");
  });

  it("写入类命令的调用名与参数固定", async () => {
    await setSetting("Settings.Language", "ja");
    expect(invokeMock).toHaveBeenLastCalledWith("set_setting", {
      key: "Settings.Language",
      value: "ja",
    });

    await saveSelectedGamePath("C:\\Games");
    expect(invokeMock).toHaveBeenLastCalledWith("set_selected_game_path", {
      path: "C:\\Games",
    });

    await saveCustomGamePaths([
      { displayName: "Steam", path: "C:\\Steam" },
    ]);
    expect(invokeMock).toHaveBeenLastCalledWith("save_custom_game_paths", {
      paths: [{ displayName: "Steam", path: "C:\\Steam" }],
    });

    await saveCustomInputMethods([
      { displayName: "自定义", category: "Japanese" },
    ]);
    expect(invokeMock).toHaveBeenLastCalledWith("save_custom_input_methods", {
      inputMethods: [{ displayName: "自定义", category: "Japanese" }],
    });
  });

  it("扫描与写入命令的参数结构符合后端契约", async () => {
    await scanInputMethods();
    expect(invokeMock).toHaveBeenLastCalledWith("scan_input_methods");

    await writeConfigFiles("C:\\Games", [
      { displayName: "微软拼音", category: "ChineseSimplified" },
    ]);
    expect(invokeMock).toHaveBeenLastCalledWith("write_config_files", {
      gameRoot: "C:\\Games",
      inputMethods: [{ displayName: "微软拼音", category: "ChineseSimplified" }],
    });
  });

  it("列表命令原样透传后端返回值", async () => {
    const paths = [{ displayName: "Steam", path: "C:\\Steam" }];
    const methods = [{ displayName: "自定义", category: "Japanese" }];
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "list_custom_game_paths") return paths;
      if (cmd === "list_custom_input_methods") return methods;
      return undefined;
    });

    await expect(listCustomGamePaths()).resolves.toEqual(paths);
    await expect(listCustomInputMethods()).resolves.toEqual(methods);
  });

  it("后端拒绝时把错误透传给调用方", async () => {
    invokeMock.mockRejectedValue({
      code: "ErrorBinVersionDirectoryNotFound",
    });

    await expect(writeConfigFiles("C:\\Games", [])).rejects.toEqual({
      code: "ErrorBinVersionDirectoryNotFound",
    });
  });
});
