import i18next from "i18next";
import { initReactI18next } from "react-i18next";

import ja from "./locales/ja.json";
import zhHans from "./locales/zh-Hans.json";
import zhHant from "./locales/zh-Hant.json";

export const LANGUAGE_AUTO = "auto";
export const LANGUAGE_ZH_HANS = "zh-Hans";
export const LANGUAGE_ZH_HANT = "zh-Hant";
export const LANGUAGE_JA = "ja";
export const LANGUAGE_PREFERENCES = [
  LANGUAGE_AUTO,
  LANGUAGE_ZH_HANS,
  LANGUAGE_ZH_HANT,
  LANGUAGE_JA,
];

/** 资源键为扁平点号命名（与原版 resw 对齐），禁用 i18next 的分隔符解析。 */
export function normalizePreference(
  preference: string | null | undefined,
): string {
  if (
    preference === LANGUAGE_ZH_HANS ||
    preference === LANGUAGE_ZH_HANT ||
    preference === LANGUAGE_JA
  ) {
    return preference;
  }
  return LANGUAGE_AUTO;
}

/**
 * auto 模式按系统语言推断：
 * - 繁体变体（zh-Hant / zh-TW / zh-HK / zh-MO）判为繁体中文
 * - 其余中文变体（zh-Hans / zh-CN / zh-SG）判为简体中文
 * - 日文（ja / ja-JP）判为日语
 * - 其余一律回退简体中文（与原版规则一致）。
 */
export function resolveSystemLanguage(): string {
  const languages = [
    ...(navigator.languages ?? []),
    navigator.language,
  ].filter(Boolean);

  for (const raw of languages) {
    const language = raw.toLowerCase();
    if (
      language.startsWith("zh-hant") ||
      language.startsWith("zh-tw") ||
      language.startsWith("zh-hk") ||
      language.startsWith("zh-mo")
    ) {
      return LANGUAGE_ZH_HANT;
    }
    if (
      language.startsWith("zh-hans") ||
      language.startsWith("zh-cn") ||
      language.startsWith("zh-sg")
    ) {
      return LANGUAGE_ZH_HANS;
    }
    if (language.startsWith("ja")) {
      return LANGUAGE_JA;
    }
  }
  return LANGUAGE_ZH_HANS;
}

export function resolveLanguage(
  preference: string | null | undefined,
): string {
  const normalized = normalizePreference(preference);
  return normalized === LANGUAGE_AUTO
    ? resolveSystemLanguage()
    : normalized;
}

let initialized = false;

export async function initI18n(preference?: string | null): Promise<string> {
  const language = resolveLanguage(preference);
  if (!initialized) {
    await i18next.use(initReactI18next).init({
      resources: {
        "zh-Hans": { translation: zhHans },
        "zh-Hant": { translation: zhHant },
        ja: { translation: ja },
      },
      lng: language,
      fallbackLng: "zh-Hans",
      keySeparator: false,
      nsSeparator: false,
      interpolation: {
        // 资源文案沿用原版 {0} 占位风格；内容由 React 转义，无需二次转义。
        prefix: "{",
        suffix: "}",
        escapeValue: false,
      },
      returnEmptyString: false,
    });
    initialized = true;
  } else {
    await i18next.changeLanguage(language);
  }
  return language;
}
