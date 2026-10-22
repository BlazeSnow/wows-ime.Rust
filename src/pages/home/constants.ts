import type { ImeCategory } from "../../lib/api";

/** 预设游戏根目录：Steam（国际服）。 */
export const STEAM_DEFAULT_PATH =
  "C:\\Program Files (x86)\\Steam\\steamapps\\common\\World of Warships";

/** 预设游戏根目录：Lesta（俄服）。 */
export const LESTA_DEFAULT_PATH = "C:\\Games\\Korabli";

/** 预设游戏根目录：360（国服）。 */
export const CN360_DEFAULT_PATH = "C:\\Games\\World_of_Warships_CN360";

/** 向导总步数：游戏路径 → 输入法选择 → 确认写入。 */
export const TOTAL_STEPS = 3;

/** 输入法分类对应的本地化键。 */
export const CATEGORY_KEYS: Record<ImeCategory, string> = {
  ChineseSimplified: "Category/ChineseSimplified",
  ChineseTraditional: "Category/ChineseTraditional",
  Japanese: "Category/Japanese",
};
