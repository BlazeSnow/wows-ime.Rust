import { beforeEach, describe, expect, it } from "vitest";

import {
  LANGUAGE_AUTO,
  LANGUAGE_JA,
  LANGUAGE_ZH_HANS,
  LANGUAGE_ZH_HANT,
  normalizePreference,
  resolveLanguage,
} from "./i18n";

function stubLanguages(languages: string[]) {
  Object.defineProperty(window.navigator, "languages", {
    value: languages,
    configurable: true,
  });
  Object.defineProperty(window.navigator, "language", {
    value: languages[0] ?? "",
    configurable: true,
  });
}

describe("normalizePreference", () => {
  it("只接受受支持的显式语言", () => {
    expect(normalizePreference(LANGUAGE_ZH_HANS)).toBe(LANGUAGE_ZH_HANS);
    expect(normalizePreference(LANGUAGE_ZH_HANT)).toBe(LANGUAGE_ZH_HANT);
    expect(normalizePreference(LANGUAGE_JA)).toBe(LANGUAGE_JA);
  });

  it("未知值与空值回退 auto", () => {
    expect(normalizePreference("en-US")).toBe(LANGUAGE_AUTO);
    expect(normalizePreference(null)).toBe(LANGUAGE_AUTO);
    expect(normalizePreference(undefined)).toBe(LANGUAGE_AUTO);
  });
});

describe("resolveLanguage", () => {
  beforeEach(() => {
    stubLanguages(["en-US"]);
  });

  it("显式模式原样返回", () => {
    expect(resolveLanguage(LANGUAGE_ZH_HANT)).toBe(LANGUAGE_ZH_HANT);
    expect(resolveLanguage(LANGUAGE_JA)).toBe(LANGUAGE_JA);
  });

  it("auto 按系统语言推断繁体中文", () => {
    stubLanguages(["zh-TW"]);
    expect(resolveLanguage(LANGUAGE_AUTO)).toBe(LANGUAGE_ZH_HANT);
  });

  it("auto 按系统语言推断简体中文", () => {
    stubLanguages(["zh-CN"]);
    expect(resolveLanguage(LANGUAGE_AUTO)).toBe(LANGUAGE_ZH_HANS);
  });

  it("auto 按系统语言推断日语", () => {
    stubLanguages(["ja-JP"]);
    expect(resolveLanguage(LANGUAGE_AUTO)).toBe(LANGUAGE_JA);
  });

  it("auto 遇到其他语言回退简体中文", () => {
    stubLanguages(["en-US"]);
    expect(resolveLanguage(LANGUAGE_AUTO)).toBe(LANGUAGE_ZH_HANS);
  });
});
