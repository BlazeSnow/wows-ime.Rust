import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

import {
  type ImeCategory,
  listCustomInputMethods,
  saveCustomInputMethods,
  scanInputMethods,
} from "../../lib/api";
import { formatWarning } from "../../lib/errors";
import type { ImeItem } from "./types";

/** 输入法列表相关的状态与操作。 */
export interface UseInputMethods {
  /** 全部输入法（系统扫描结果 + 自定义）。 */
  inputMethods: ImeItem[];
  /** 已勾选的输入法。 */
  selectedInputMethods: ImeItem[];
  /** 待确认删除的输入法。 */
  pendingDeletion: ImeItem | null;
  toggleInputMethod: (displayName: string, checked: boolean) => void;
  changeCategory: (displayName: string, category: ImeCategory) => void;
  addCustomInputMethod: (displayName: string, category: ImeCategory) => void;
  requestDeletion: (item: ImeItem) => void;
  cancelDeletion: () => void;
  confirmDeletion: () => void;
  /** 重新扫描系统输入法。 */
  rescan: () => void;
  /** 进入第 2 步时报告扫描结果（只报一次）。 */
  reportScanOutcome: () => Promise<void>;
  /** 立即把自定义输入法写库（写入配置后调用，确保分类已落盘）。 */
  persistCustomInputMethods: (items: ImeItem[]) => void;
}

/** 输入法扫描、分类调整与自定义项的增删。 */
export function useInputMethods(): UseInputMethods {
  const { t } = useTranslation();
  const [inputMethods, setInputMethods] = useState<ImeItem[]>([]);
  const [pendingDeletion, setPendingDeletion] = useState<ImeItem | null>(null);

  // 预扫描只执行一次（StrictMode 下 effect 会重复执行）
  const initialScanStarted = useRef(false);
  // 启动预扫描的 promise：进入第 2 步时等它完成，避免扫描未完就提示
  const scanPromise = useRef<Promise<void> | null>(null);
  // 最近一次扫描的结果，供进入第 2 步时提示
  const scanOutcome = useRef<{ count: number; warning: string } | null>(null);
  // 第 2 步的扫描提示只报一次
  const scanNotified = useRef(false);

  const persistCustomInputMethods = useCallback((items: ImeItem[]) => {
    void saveCustomInputMethods(
      items
        .filter((item) => item.isCustom)
        .map((item) => ({
          displayName: item.displayName,
          category: item.category,
        })),
    ).catch(() => undefined);
  }, []);

  // 载入已保存的自定义输入法，追加到扫描结果之后。
  const loadSavedCustomInputMethods = useCallback(async () => {
    const saved = (await listCustomInputMethods().catch(() => [])) ?? [];
    setInputMethods((current) => {
      const existing = new Set(
        current.map((item) => item.displayName.toLowerCase()),
      );
      const additions: ImeItem[] = saved
        .filter(
          (item) =>
            item.displayName.trim().length > 0 &&
            !existing.has(item.displayName.toLowerCase()),
        )
        .map((item) => ({
          displayName: item.displayName,
          category: item.category as ImeCategory,
          isCustom: true,
          isSelected: false,
        }));
      return [...current, ...additions];
    });
  }, []);

  /**
   * 扫描输入法。
   *
   * `silent` 为真时不弹提示——启动时的预扫描属于「准备数据」，
   * 此时用户还在第 1 步，输入法列表尚未可见，提示时机不对；
   * 真正的提示在进入第 2 步时给出（见 `reportScanOutcome`）。
   */
  const scanInputMethodList = useCallback(
    async (silent = false) => {
      const result = await scanInputMethods().catch(() => null);
      if (!result) {
        if (!silent) {
          toast.error(t("Status/ScanCompletedNoImeWithWarning", { 0: "" }));
        }
        return;
      }

      const warning = result.warningCode
        ? formatWarning(t, result.warningCode, result.warningArguments)
        : "";
      const scanned: ImeItem[] = result.candidates.map((candidate) => ({
        displayName: candidate.displayName,
        category: candidate.category,
        isCustom: false,
        isSelected: false,
      }));

      setInputMethods((current) => {
        const customs = current.filter((item) => item.isCustom);
        return [...scanned, ...customs];
      });
      scanOutcome.current = { count: scanned.length, warning };

      if (silent) {
        return;
      }
      if (scanned.length === 0 && warning) {
        toast.warning(t("Status/ScanCompletedNoImeWithWarning", { 0: warning }));
        return;
      }
      toast.success(t("Status/ScanCompletedWithCount", { 0: scanned.length }));
    },
    [t],
  );

  useEffect(() => {
    // 预扫描只做一次：StrictMode 下 effect 会重复执行，重复扫描既浪费也会弹两次提示
    if (initialScanStarted.current) {
      return;
    }
    initialScanStarted.current = true;
    scanPromise.current = scanInputMethodList(true).then(() =>
      loadSavedCustomInputMethods(),
    );
  }, [scanInputMethodList, loadSavedCustomInputMethods]);

  const toggleInputMethod = useCallback(
    (displayName: string, checked: boolean) => {
      setInputMethods((current) =>
        current.map((item) =>
          item.displayName === displayName
            ? { ...item, isSelected: checked }
            : item,
        ),
      );
    },
    [],
  );

  const changeCategory = useCallback(
    (displayName: string, category: ImeCategory) => {
      setInputMethods((current) => {
        const next = current.map((item) =>
          item.displayName === displayName ? { ...item, category } : item,
        );
        persistCustomInputMethods(next);
        return next;
      });
    },
    [persistCustomInputMethods],
  );

  const addCustomInputMethod = useCallback(
    (displayName: string, category: ImeCategory) => {
      const name = displayName.trim();
      if (name.length === 0) {
        toast.warning(t("Status/ImeNameEmpty"));
        return;
      }
      if (
        inputMethods.some(
          (item) => item.displayName.toLowerCase() === name.toLowerCase(),
        )
      ) {
        toast.warning(t("Status/ImeNameExists"));
        return;
      }
      const next = [
        ...inputMethods,
        { displayName: name, category, isCustom: true, isSelected: true },
      ];
      setInputMethods(next);
      persistCustomInputMethods(next);
      toast.success(t("Status/CustomImeAdded"));
    },
    [inputMethods, persistCustomInputMethods, t],
  );

  const confirmDeletion = useCallback(() => {
    if (!pendingDeletion) {
      return;
    }
    const next = inputMethods.filter(
      (item) => item.displayName !== pendingDeletion.displayName,
    );
    setInputMethods(next);
    persistCustomInputMethods(next);
    setPendingDeletion(null);
    toast.success(t("Status/CustomImeDeleted"));
  }, [inputMethods, pendingDeletion, persistCustomInputMethods, t]);

  const rescan = useCallback(() => {
    void scanInputMethodList().then(() => loadSavedCustomInputMethods());
  }, [loadSavedCustomInputMethods, scanInputMethodList]);

  const reportScanOutcome = useCallback(async () => {
    await scanPromise.current;
    if (scanNotified.current) {
      return;
    }
    scanNotified.current = true;
    const outcome = scanOutcome.current;
    if (!outcome) {
      return;
    }
    if (outcome.count === 0 && outcome.warning) {
      toast.warning(
        t("Status/ScanCompletedNoImeWithWarning", { 0: outcome.warning }),
      );
      return;
    }
    toast.success(t("Status/ScanCompletedWithCount", { 0: outcome.count }));
  }, [t]);

  const selectedInputMethods = useMemo(
    () => inputMethods.filter((item) => item.isSelected),
    [inputMethods],
  );

  return {
    inputMethods,
    selectedInputMethods,
    pendingDeletion,
    toggleInputMethod,
    changeCategory,
    addCustomInputMethod,
    requestDeletion: setPendingDeletion,
    cancelDeletion: () => setPendingDeletion(null),
    confirmDeletion,
    rescan,
    reportScanOutcome,
    persistCustomInputMethods,
  };
}
