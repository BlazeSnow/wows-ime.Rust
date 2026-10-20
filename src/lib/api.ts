import { invoke } from "@tauri-apps/api/core";

/** 输入法分类，取值与后端 `ImeCategory` 一致。 */
export type ImeCategory = "ChineseSimplified" | "ChineseTraditional" | "Japanese";

/** 扫描到的输入法候选项。 */
export interface ScannedImeCandidate {
  displayName: string;
  category: ImeCategory;
  confidence: number;
}

/** 扫描结果；`warningCode` 非空表示扫描未完全成功。 */
export interface InputMethodScanResult {
  candidates: ScannedImeCandidate[];
  warningCode: string | null;
  warningArguments: string[] | null;
}

/** 写入配置文件所需的输入法定义。 */
export interface InputMethodDefinition {
  displayName: string;
  category: ImeCategory;
}

/** 自定义游戏路径。 */
export interface PersistedGamePath {
  displayName: string;
  path: string;
}

/** 自定义输入法。 */
export interface PersistedInputMethod {
  displayName: string;
  category: string;
}

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

/** 扫描系统已启用的中文/日文输入法。 */
export function scanInputMethods(): Promise<InputMethodScanResult> {
  return invoke<InputMethodScanResult>("scan_input_methods");
}

/** 游戏根目录是否包含游戏可执行文件。 */
export function hasGameExecutable(gameRoot: string): Promise<boolean> {
  return invoke<boolean>("has_game_executable", { gameRoot });
}

/** 需要写入的配置文件路径列表。 */
export function resolveTargetConfigFiles(gameRoot: string): Promise<string[]> {
  return invoke<string[]>("resolve_target_config_files", { gameRoot });
}

/** 目标路径中已存在配置文件的数量。 */
export function existingConfigCount(gameRoot: string): Promise<number> {
  return invoke<number>("existing_config_count", { gameRoot });
}

/** 写入输入法配置文件；返回实际写入的文件数量。 */
export function writeConfigFiles(
  gameRoot: string,
  inputMethods: InputMethodDefinition[],
): Promise<number> {
  return invoke<number>("write_config_files", { gameRoot, inputMethods });
}

/** 读取全部自定义游戏路径。 */
export function listCustomGamePaths(): Promise<PersistedGamePath[]> {
  return invoke<PersistedGamePath[]>("list_custom_game_paths");
}

/** 全量同步自定义游戏路径。 */
export function saveCustomGamePaths(paths: PersistedGamePath[]): Promise<void> {
  return invoke<void>("save_custom_game_paths", { paths });
}

/** 读取全部自定义输入法。 */
export function listCustomInputMethods(): Promise<PersistedInputMethod[]> {
  return invoke<PersistedInputMethod[]>("list_custom_input_methods");
}

/** 全量同步自定义输入法。 */
export function saveCustomInputMethods(
  inputMethods: PersistedInputMethod[],
): Promise<void> {
  return invoke<void>("save_custom_input_methods", { inputMethods });
}

/** 当前选中的游戏根目录。 */
export function getSelectedGamePath(): Promise<string> {
  return invoke<string>("get_selected_game_path");
}

/** 保存当前选中的游戏根目录。 */
export function saveSelectedGamePath(path: string): Promise<void> {
  return invoke<void>("set_selected_game_path", { path });
}
