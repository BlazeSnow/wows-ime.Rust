import { invoke } from "@tauri-apps/api/core";

/** 应用版本号（取自 Cargo.toml 的 package.version）。 */
export function getAppVersion(): Promise<string> {
  return invoke<string>("get_app_version");
}

/** 应用数据目录路径。 */
export function getDataDir(): Promise<string> {
  return invoke<string>("get_data_dir");
}

/** 界面语言模式（auto / zh-Hans / zh-Hant / ja）。 */
export function getLanguagePreference(): Promise<string> {
  return invoke<string>("get_language_preference");
}

/** 读取简单设置项；不存在时返回空串。 */
export function getSetting(key: string): Promise<string> {
  return invoke<string>("get_setting", { key });
}

/** 写入简单设置项。 */
export function setSetting(key: string, value: string): Promise<void> {
  return invoke<void>("set_setting", { key, value });
}
