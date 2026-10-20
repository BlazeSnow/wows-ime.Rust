import i18next from "i18next";
import { beforeAll, describe, expect, it } from "vitest";

import { initI18n } from "./i18n";

describe("本地化插值", () => {
  beforeAll(async () => {
    await initI18n("zh-Hans");
  });

  it("用 {0} 占位符替换参数", () => {
    // 资源文案沿用原版 {0} 占位风格，i18next 的分隔符已改为单花括号。
    const rendered = i18next.t("Status/WriteSucceededWithCount", { 0: 3 });
    expect(rendered).toContain("3");
  });

  it("多占位符按序替换", () => {
    const rendered = i18next.t("Step/Progress", { 0: 2, 1: 3 });
    expect(rendered).toContain("2");
    expect(rendered).toContain("3");
  });

  it("未知键回退到键名本身", () => {
    expect(i18next.t("Not/Existing/Key")).toBe("Not/Existing/Key");
  });
});
