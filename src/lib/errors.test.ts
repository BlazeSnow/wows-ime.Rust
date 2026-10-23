import { describe, expect, it } from "vitest";

import { describeError, formatWarning } from "./errors";

describe("describeError", () => {
  it("字符串错误原样返回", () => {
    expect(describeError("磁盘已满")).toBe("磁盘已满");
  });

  it("有 detail 时只展示 detail，不带错误码前缀", () => {
    expect(
      describeError({
        code: "ErrorWriteConfigFailed",
        detail: "C:\\Games\\bin\\1\\res_mods\\ime_config.xml: 拒绝访问",
      }),
    ).toBe("C:\\Games\\bin\\1\\res_mods\\ime_config.xml: 拒绝访问");
  });

  it("没有 detail 时回退到错误码", () => {
    expect(describeError({ code: "ErrorBinVersionDirectoryNotFound" })).toBe(
      "ErrorBinVersionDirectoryNotFound",
    );
  });

  it("无法识别的错误转成字符串", () => {
    expect(describeError(42)).toBe("42");
    expect(describeError(null)).toBe("null");
    expect(describeError(undefined)).toBe("undefined");
  });
});

describe("formatWarning", () => {
  const t = ((key: string, options?: Record<string, unknown>) => {
    const table: Record<string, string> = {
      "Tsf/CoInitializeFailed": `CoInitializeEx 失败: ${options?.[0] ?? ""}`,
      "Tsf/GetLanguageListEmpty": "GetLanguageList 返回空语言列表",
    };
    return table[key] ?? key;
  }) as unknown as Parameters<typeof formatWarning>[0];

  it("用本地化文案替换警告码", () => {
    expect(formatWarning(t, "Tsf/GetLanguageListEmpty", null)).toBe(
      "GetLanguageList 返回空语言列表",
    );
  });

  it("按位置填充警告参数", () => {
    expect(formatWarning(t, "Tsf/CoInitializeFailed", ["0x80010106"])).toBe(
      "CoInitializeEx 失败: 0x80010106",
    );
  });

  it("资源缺失时回退显示原始警告码", () => {
    expect(formatWarning(t, "Tsf/UnknownCode", null)).toBe("Tsf/UnknownCode");
  });
});
