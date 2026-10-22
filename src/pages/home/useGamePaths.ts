import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

import {
  getSelectedGamePath,
  listCustomGamePaths,
  saveCustomGamePaths,
  saveSelectedGamePath,
} from "../../lib/api";
import { describeError } from "../../lib/errors";
import {
  CN360_DEFAULT_PATH,
  LESTA_DEFAULT_PATH,
  STEAM_DEFAULT_PATH,
} from "./constants";
import type { GamePathOption } from "./types";

/** 游戏路径相关的状态与操作。 */
export interface UseGamePaths {
  /** 全部路径选项（预设 + 自定义）。 */
  gamePaths: GamePathOption[];
  /** 当前选中的路径。 */
  selectedGamePath: string;
  /** 待确认删除的路径。 */
  pendingDeletion: GamePathOption | null;
  selectGamePath: (path: string) => void;
  addCustomGamePath: () => Promise<void>;
  openGameFolder: () => Promise<void>;
  requestDeletion: (option: GamePathOption) => void;
  cancelDeletion: () => void;
  confirmDeletion: () => void;
  /** 立即把当前选中路径写库（写入配置后调用，确保选择已落盘）。 */
  persistSelection: (path: string) => void;
}

/** 游戏路径的加载、选择、增删与打开目录。 */
export function useGamePaths(): UseGamePaths {
  const { t } = useTranslation();
  const [gamePaths, setGamePaths] = useState<GamePathOption[]>([]);
  const [selectedGamePath, setSelectedGamePath] = useState("");
  const [pendingDeletion, setPendingDeletion] = useState<GamePathOption | null>(
    null,
  );

  const persistSelectedGamePath = useCallback(
    (path: string) => {
      void saveSelectedGamePath(path).catch((error) => {
        // 静默失败会让人以为保存成功、下次启动才发现没了，必须提示
        console.error("保存选中的游戏路径失败", error);
        toast.error(t("Status/SaveFailed", { 0: describeError(error) }));
      });
    },
    [t],
  );

  const persistCustomGamePaths = useCallback(
    (options: GamePathOption[]) => {
      void saveCustomGamePaths(
        options
          .filter((option) => option.isCustom)
          .map((option) => ({
            displayName: option.displayName,
            path: option.path,
          })),
      ).catch((error) => {
        console.error("保存自定义游戏路径失败", error);
        toast.error(t("Status/SaveFailed", { 0: describeError(error) }));
      });
    },
    [t],
  );

  // 载入游戏路径选项（预设 + 自定义），并恢复上次选择。
  useEffect(() => {
    void (async () => {
      const [savedPaths, savedSelection] = await Promise.all([
        // catch 只兜 reject；后端若返回了非数组也按空列表处理，避免列表渲染时崩掉
        listCustomGamePaths()
          .catch(() => [])
          .then((paths) => paths ?? []),
        getSelectedGamePath().catch(() => ""),
      ]);

      const presets: GamePathOption[] = [
        {
          displayName: t("GamePathOption/Steam"),
          path: STEAM_DEFAULT_PATH,
          isCustom: false,
        },
        {
          displayName: t("GamePathOption/Lesta"),
          path: LESTA_DEFAULT_PATH,
          isCustom: false,
        },
        {
          displayName: t("GamePathOption/Cn360"),
          path: CN360_DEFAULT_PATH,
          isCustom: false,
        },
      ];

      const customPaths: GamePathOption[] = savedPaths
        .map((saved) => ({
          displayName: saved.displayName || saved.path,
          path: saved.path,
          isCustom: true,
        }))
        .filter(
          (option) =>
            option.path.trim().length > 0 &&
            !presets.some(
              (preset) =>
                preset.path.toLowerCase() === option.path.toLowerCase(),
            ),
        );

      const all = [...presets, ...customPaths];
      const restored =
        all.find(
          (option) =>
            option.path.toLowerCase() === (savedSelection ?? "").toLowerCase(),
        ) ?? all[0];
      setGamePaths(all);
      setSelectedGamePath(restored?.path ?? "");
    })();
  }, [t]);

  const selectGamePath = useCallback(
    (path: string) => {
      setSelectedGamePath(path);
      persistSelectedGamePath(path);
    },
    [persistSelectedGamePath],
  );

  const addCustomGamePath = useCallback(async () => {
    const picked = await open({ directory: true, multiple: false }).catch(
      () => null,
    );
    if (typeof picked !== "string" || picked.trim().length === 0) {
      return;
    }
    const path = picked.trim();

    const existing = gamePaths.find(
      (option) => option.path.toLowerCase() === path.toLowerCase(),
    );
    if (existing) {
      selectGamePath(existing.path);
      toast.info(t("Status/CustomGamePathExists"));
      return;
    }

    const name = path.split(/[\\/]/).filter(Boolean).pop() ?? path;
    const option: GamePathOption = {
      displayName: name || path,
      path,
      isCustom: true,
    };
    const next = [...gamePaths, option];
    setGamePaths(next);
    selectGamePath(path);
    persistCustomGamePaths(next);
    toast.success(t("Status/CustomGamePathAdded"));
  }, [gamePaths, persistCustomGamePaths, selectGamePath, t]);

  const confirmDeletion = useCallback(() => {
    if (!pendingDeletion) {
      return;
    }
    const wasSelected = pendingDeletion.path === selectedGamePath;
    const next = gamePaths.filter(
      (option) => option.path !== pendingDeletion.path,
    );
    setGamePaths(next);
    persistCustomGamePaths(next);
    if (wasSelected) {
      const fallback = next[0]?.path ?? "";
      setSelectedGamePath(fallback);
      persistSelectedGamePath(fallback);
    }
    setPendingDeletion(null);
    toast.success(t("Status/CustomGamePathDeleted"));
  }, [gamePaths, pendingDeletion, persistCustomGamePaths, persistSelectedGamePath, selectedGamePath, t]);

  const openGameFolder = useCallback(async () => {
    if (!selectedGamePath) {
      return;
    }
    const opened = await openPath(selectedGamePath).catch(() => false);
    if (!opened) {
      toast.error(t("Status/OpenDirectoryFailed", { 0: selectedGamePath }));
    }
  }, [selectedGamePath, t]);

  return {
    gamePaths,
    selectedGamePath,
    pendingDeletion,
    selectGamePath,
    addCustomGamePath,
    openGameFolder,
    requestDeletion: setPendingDeletion,
    cancelDeletion: () => setPendingDeletion(null),
    confirmDeletion,
    persistSelection: persistSelectedGamePath,
  };
}
