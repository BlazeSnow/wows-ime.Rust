import type { ImeCategory } from "../../lib/api";

/** 游戏路径选项：预设路径或用户添加的自定义路径。 */
export interface GamePathOption {
  displayName: string;
  path: string;
  isCustom: boolean;
}

/** 输入法列表项。 */
export interface ImeItem {
  displayName: string;
  category: ImeCategory;
  /** 是否为用户手动添加（系统扫描到的为 false）。 */
  isCustom: boolean;
  isSelected: boolean;
}
