import { open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "../components/ui/alert-dialog";
import { Badge } from "../components/ui/badge";
import { Button } from "../components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { Checkbox } from "../components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../components/ui/dialog";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../components/ui/select";
import { Separator } from "../components/ui/separator";
import {
  type ImeCategory,
  type InputMethodDefinition,
  existingConfigCount,
  getSelectedGamePath,
  hasGameExecutable,
  listCustomGamePaths,
  listCustomInputMethods,
  resolveTargetConfigFiles,
  saveCustomGamePaths,
  saveCustomInputMethods,
  scanInputMethods,
  saveSelectedGamePath,
  writeConfigFiles,
} from "../lib/api";

const STEAM_DEFAULT_PATH =
  "C:\\Program Files (x86)\\Steam\\steamapps\\common\\World of Warships";
const LESTA_DEFAULT_PATH = "C:\\Games\\Korabli";
const CN360_DEFAULT_PATH = "C:\\Games\\World_of_Warships_CN360";
const TOTAL_STEPS = 3;

const CATEGORY_KEYS: Record<ImeCategory, string> = {
  ChineseSimplified: "Category/ChineseSimplified",
  ChineseTraditional: "Category/ChineseTraditional",
  Japanese: "Category/Japanese",
};

interface GamePathOption {
  displayName: string;
  path: string;
  isCustom: boolean;
}

interface ImeItem {
  displayName: string;
  category: ImeCategory;
  isCustom: boolean;
  isSelected: boolean;
}

/** 主页面：游戏路径 → 输入法选择 → 确认写入 三步向导。 */
export function HomePage() {
  const { t } = useTranslation();
  const [step, setStep] = useState(1);
  const [gamePaths, setGamePaths] = useState<GamePathOption[]>([]);
  const [selectedGamePath, setSelectedGamePath] = useState("");
  const [inputMethods, setInputMethods] = useState<ImeItem[]>([]);
  const [pendingDeletion, setPendingDeletion] = useState<GamePathOption | null>(
    null,
  );
  const [pendingImeDeletion, setPendingImeDeletion] = useState<ImeItem | null>(
    null,
  );
  const [addImeOpen, setAddImeOpen] = useState(false);
  const [writeConfirm, setWriteConfirm] = useState<{
    open: boolean;
    existing: number;
    total: number;
  }>({ open: false, existing: 0, total: 0 });

  const selectedOptions = useMemo(
    () => gamePaths.filter((option) => option.path === selectedGamePath),
    [gamePaths, selectedGamePath],
  );
  const selectedInputMethods = useMemo(
    () => inputMethods.filter((item) => item.isSelected),
    [inputMethods],
  );

  // 载入游戏路径选项（预设 + 自定义），并恢复上次选择。
  const loadGamePaths = useCallback(async () => {
    const [savedPaths, savedSelection] = await Promise.all([
      listCustomGamePaths().catch(() => []),
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
            (preset) => preset.path.toLowerCase() === option.path.toLowerCase(),
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
  }, [t]);

  // 载入已保存的自定义输入法，追加到扫描结果之后。
  const loadSavedCustomInputMethods = useCallback(async () => {
    const saved = await listCustomInputMethods().catch(() => []);
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

  const scanInputMethodList = useCallback(async () => {
    const result = await scanInputMethods().catch(() => null);
    if (!result) {
      toast.error(t("Status/ScanCompletedNoImeWithWarning", { 0: "" }));
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

    if (scanned.length === 0 && warning) {
      toast.warning(t("Status/ScanCompletedNoImeWithWarning", { 0: warning }));
      return;
    }
    toast.success(t("Status/ScanCompletedWithCount", { 0: scanned.length }));
  }, [t]);

  useEffect(() => {
    void loadGamePaths();
    void scanInputMethodList().then(loadSavedCustomInputMethods);
  }, [loadGamePaths, scanInputMethodList, loadSavedCustomInputMethods]);

  const persistSelectedGamePath = (path: string) => {
    void saveSelectedGamePath(path).catch(() => undefined);
  };

  const persistCustomGamePaths = (options: GamePathOption[]) => {
    void saveCustomGamePaths(
      options
        .filter((option) => option.isCustom)
        .map((option) => ({
          displayName: option.displayName,
          path: option.path,
        })),
    ).catch(() => undefined);
  };

  const persistCustomInputMethods = (items: ImeItem[]) => {
    void saveCustomInputMethods(
      items
        .filter((item) => item.isCustom)
        .map((item) => ({
          displayName: item.displayName,
          category: item.category,
        })),
    ).catch(() => undefined);
  };

  const selectGamePath = (path: string) => {
    setSelectedGamePath(path);
    persistSelectedGamePath(path);
  };

  const addCustomGamePath = async () => {
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
  };

  const deleteCustomGamePath = () => {
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
  };

  const openGameFolder = async () => {
    if (!selectedGamePath) {
      return;
    }
    const opened = await openPath(selectedGamePath).catch(() => false);
    if (!opened) {
      toast.error(t("Status/OpenDirectoryFailed", { 0: selectedGamePath }));
    }
  };

  const toggleInputMethod = (displayName: string, checked: boolean) => {
    setInputMethods((current) =>
      current.map((item) =>
        item.displayName === displayName
          ? { ...item, isSelected: checked }
          : item,
      ),
    );
  };

  const changeCategory = (displayName: string, category: ImeCategory) => {
    setInputMethods((current) => {
      const next = current.map((item) =>
        item.displayName === displayName ? { ...item, category } : item,
      );
      persistCustomInputMethods(next);
      return next;
    });
  };

  const deleteCustomInputMethod = () => {
    if (!pendingImeDeletion) {
      return;
    }
    const next = inputMethods.filter(
      (item) => item.displayName !== pendingImeDeletion.displayName,
    );
    setInputMethods(next);
    persistCustomInputMethods(next);
    setPendingImeDeletion(null);
    toast.success(t("Status/CustomImeDeleted"));
  };

  const addCustomInputMethod = (displayName: string, category: ImeCategory) => {
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
  };

  const goNext = async () => {
    if (step === 1 && selectedGamePath.length === 0) {
      toast.warning(t("Status/GameRootInvalid"));
      return;
    }
    if (step === 2 && selectedInputMethods.length === 0) {
      toast.warning(t("Status/SelectAtLeastOneIme"));
      return;
    }
    if (step < TOTAL_STEPS) {
      setStep(step + 1);
    }
  };

  const prepareWrite = async () => {
    if (selectedInputMethods.length === 0) {
      toast.warning(t("Status/SelectAtLeastOneIme"));
      return;
    }
    if (selectedGamePath.length === 0) {
      toast.error(t("Status/GameRootInvalid"));
      return;
    }

    const hasExe = await hasGameExecutable(selectedGamePath).catch(() => false);
    if (!hasExe) {
      toast.error(t("Status/GameExeNotFound"));
      return;
    }

    const targets = await resolveTargetConfigFiles(selectedGamePath).catch(
      () => [] as string[],
    );
    if (targets.length === 0) {
      toast.error(t("Status/BinVersionDirectoryNotFound"));
      return;
    }

    const existing = await existingConfigCount(selectedGamePath).catch(
      () => 0,
    );
    setWriteConfirm({ open: true, existing, total: targets.length });
  };

  const confirmWrite = async () => {
    setWriteConfirm((current) => ({ ...current, open: false }));
    const definitions: InputMethodDefinition[] = selectedInputMethods.map(
      (item) => ({ displayName: item.displayName, category: item.category }),
    );
    try {
      const written = await writeConfigFiles(selectedGamePath, definitions);
      toast.success(t("Status/WriteSucceededWithCount", { 0: written }));
      persistSelectedGamePath(selectedGamePath);
      persistCustomInputMethods(inputMethods);
    } catch (error) {
      toast.error(t("Status/WriteFailed", { 0: describeError(error) }));
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <span className="text-sm text-muted-foreground">
          {t("Step/Progress", { 0: step, 1: TOTAL_STEPS })}
        </span>
        <Separator className="flex-1" />
        {step > 1 && (
          <Button variant="outline" onClick={() => setStep(step - 1)}>
            {t("Step/Previous")}
          </Button>
        )}
        {step < TOTAL_STEPS ? (
          <Button onClick={() => void goNext()}>{t("Step/Next")}</Button>
        ) : (
          <Button onClick={() => void prepareWrite()}>
            {t("WriteConfigButton.Content")}
          </Button>
        )}
      </div>

      {step === 1 && (
        <Card>
          <CardHeader>
            <CardTitle>{t("GameRootLabel.Text")}</CardTitle>
            <CardDescription>
              {t("GamePathSectionHint.Text")}
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            <div className="flex flex-col gap-2">
              {gamePaths.map((option) => (
                <label
                  key={option.path}
                  className="flex items-center gap-3 rounded-lg border px-3 py-2 text-sm"
                >
                  <input
                    type="radio"
                    name="game-path"
                    className="size-4"
                    checked={option.path === selectedGamePath}
                    onChange={() => selectGamePath(option.path)}
                  />
                  <span className="font-medium">{option.displayName}</span>
                  <span className="min-w-0 flex-1 truncate text-muted-foreground">
                    {option.path}
                  </span>
                  {option.isCustom && (
                    <Button
                      variant="ghost"
                      size="xs"
                      onClick={() => setPendingDeletion(option)}
                    >
                      {t("DeleteCustomGamePathButtonTemplate.Content")}
                    </Button>
                  )}
                </label>
              ))}
            </div>
            <div className="flex gap-2">
              <Button variant="outline" onClick={() => void addCustomGamePath()}>
                {t("AddCustomGamePathButton.Content")}
              </Button>
              <Button
                variant="outline"
                disabled={selectedGamePath.length === 0}
                onClick={() => void openGameFolder()}
              >
                {t("OpenFolderButton.Content")}
              </Button>
            </div>
            {selectedOptions.length > 0 && (
              <p className="text-xs text-muted-foreground">
                {selectedOptions[0].path}
              </p>
            )}
          </CardContent>
        </Card>
      )}

      {step === 2 && (
        <Card>
          <CardHeader>
            <CardTitle>{t("ImeSectionTitle.Text")}</CardTitle>
            <CardDescription>{t("GamePathSectionHint.Text")}</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            <div className="grid grid-cols-[1fr_10rem_6rem] gap-2 text-xs text-muted-foreground">
              <span>{t("ImeTableHeaderName.Text")}</span>
              <span>{t("ImeTableHeaderCategory.Text")}</span>
              <span>{t("ImeTableHeaderAction.Text")}</span>
            </div>
            <div className="flex max-h-96 flex-col gap-1 overflow-y-auto">
              {inputMethods.map((item) => (
                <div
                  key={item.displayName}
                  className="grid grid-cols-[1fr_10rem_6rem] items-center gap-2 rounded-lg border px-3 py-1.5 text-sm"
                >
                  <label className="flex min-w-0 items-center gap-2">
                    <Checkbox
                      checked={item.isSelected}
                      onCheckedChange={(checked) =>
                        toggleInputMethod(
                          item.displayName,
                          checked === true,
                        )
                      }
                    />
                    <span className="truncate">{item.displayName}</span>
                  </label>
                  <Select
                    value={item.category}
                    onValueChange={(value) =>
                      changeCategory(item.displayName, value as ImeCategory)
                    }
                  >
                    <SelectTrigger className="w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {(
                        Object.keys(CATEGORY_KEYS) as ImeCategory[]
                      ).map((category) => (
                        <SelectItem key={category} value={category}>
                          {t(CATEGORY_KEYS[category])}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <div className="flex justify-end">
                    {item.isCustom ? (
                      <Button
                        variant="ghost"
                        size="xs"
                        onClick={() => setPendingImeDeletion(item)}
                      >
                        {t("DeleteCustomImeButtonTemplate.Content")}
                      </Button>
                    ) : (
                      <Badge variant="secondary">IME</Badge>
                    )}
                  </div>
                </div>
              ))}
            </div>
            <div className="flex gap-2">
              <Button variant="outline" onClick={() => setAddImeOpen(true)}>
                {t("AddCustomImeButton.Content")}
              </Button>
              <Button
                variant="outline"
                onClick={() => void scanInputMethodList()}
              >
                {t("RefreshImeButton.Content")}
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {step === TOTAL_STEPS && (
        <Card>
          <CardHeader>
            <CardTitle>{t("WriteConfigCardHeader.Text")}</CardTitle>
            <CardDescription>{t("GameRootLabel.Text")}</CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            <p className="text-sm">{selectedGamePath}</p>
            <Separator />
            <p className="text-sm font-medium">
              {t("ConfirmImeSectionHeader.Text")}
            </p>
            <ul className="flex flex-col gap-1 text-sm text-muted-foreground">
              {selectedInputMethods.map((item, index) => (
                <li key={item.displayName}>
                  {index + 1}. {item.displayName}
                  <span className="ml-2 text-xs">
                    {t(CATEGORY_KEYS[item.category])}
                  </span>
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      )}

      <AlertDialog
        open={pendingDeletion !== null}
        onOpenChange={(open) => {
          if (!open) {
            setPendingDeletion(null);
          }
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t("Dialog/DeleteCustomGamePath/Title")}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {pendingDeletion
                ? t("Dialog/DeleteCustomGamePath/Content", {
                    0: pendingDeletion.displayName,
                    1: pendingDeletion.path,
                  })
                : ""}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              {t("Dialog/Common/Cancel")}
            </AlertDialogCancel>
            <AlertDialogAction onClick={deleteCustomGamePath}>
              {t("Dialog/DeleteCustomGamePath/PrimaryButton")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={pendingImeDeletion !== null}
        onOpenChange={(open) => {
          if (!open) {
            setPendingImeDeletion(null);
          }
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t("Dialog/DeleteCustomIme/Title")}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {pendingImeDeletion
                ? t("Dialog/DeleteCustomIme/Content", {
                    0: pendingImeDeletion.displayName,
                  })
                : ""}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              {t("Dialog/Common/Cancel")}
            </AlertDialogCancel>
            <AlertDialogAction onClick={deleteCustomInputMethod}>
              {t("Dialog/DeleteCustomIme/PrimaryButton")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AddInputMethodDialog
        open={addImeOpen}
        onOpenChange={setAddImeOpen}
        onConfirm={addCustomInputMethod}
      />

      <AlertDialog
        open={writeConfirm.open}
        onOpenChange={(open) =>
          setWriteConfirm((current) => ({ ...current, open }))
        }
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {writeConfirm.existing > 0
                ? t("Dialog/Overwrite/Title")
                : t("Dialog/AddConfig/Title")}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {writeConfirm.existing > 0
                ? t("Dialog/Overwrite/Content", { 0: writeConfirm.existing })
                : t("Dialog/AddConfig/Content", { 0: writeConfirm.total })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              {t("Dialog/Common/Cancel")}
            </AlertDialogCancel>
            <AlertDialogAction onClick={() => void confirmWrite()}>
              {writeConfirm.existing > 0
                ? t("Dialog/Overwrite/PrimaryButton")
                : t("Dialog/AddConfig/PrimaryButton")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

interface AddInputMethodDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: (displayName: string, category: ImeCategory) => void;
}

function AddInputMethodDialog({
  open,
  onOpenChange,
  onConfirm,
}: AddInputMethodDialogProps) {
  const { t } = useTranslation();
  const [displayName, setDisplayName] = useState("");
  const [category, setCategory] = useState<ImeCategory>("ChineseSimplified");

  useEffect(() => {
    if (open) {
      setDisplayName("");
      setCategory("ChineseSimplified");
    }
  }, [open]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("Dialog/AddCustomIme/Title")}</DialogTitle>
          <DialogDescription>
            {t("Dialog/AddCustomIme/NameLabel")}
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-3">
          <Label htmlFor="custom-ime-name">{t("Dialog/AddCustomIme/NameLabel")}</Label>
          <Input
            id="custom-ime-name"
            value={displayName}
            placeholder={t("Dialog/AddCustomIme/Placeholder")}
            onChange={(event) => setDisplayName(event.target.value)}
          />
          <Label htmlFor="custom-ime-category">
            {t("Dialog/AddCustomIme/CategoryLabel")}
          </Label>
          <Select
            value={category}
            onValueChange={(value) => setCategory(value as ImeCategory)}
          >
            <SelectTrigger id="custom-ime-category" className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {(Object.keys(CATEGORY_KEYS) as ImeCategory[]).map((key) => (
                <SelectItem key={key} value={key}>
                  {t(CATEGORY_KEYS[key])}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
          >
            {t("Dialog/Common/Cancel")}
          </Button>
          <Button
            onClick={() => {
              onConfirm(displayName, category);
              onOpenChange(false);
            }}
          >
            {t("Dialog/AddCustomIme/PrimaryButton")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/** 扫描警告码格式化：优先用本地化文案，否则回退原始警告码。 */
function formatWarning(
  t: (key: string, options?: Record<string, unknown>) => string,
  code: string,
  args: string[] | null,
): string {
  const values = args ?? [];
  const options: Record<string, unknown> = {};
  values.forEach((value, index) => {
    options[index] = value;
  });
  const translated = t(code, options);
  return translated === code ? code : translated;
}

/** 提取后端错误信息：优先用可读的 detail，其次才回退到错误码。 */
function describeError(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error && typeof error === "object" && "code" in error) {
    const detail = (error as { detail?: unknown }).detail;
    return detail ? String(detail) : String((error as { code: unknown }).code);
  }
  return String(error);
}
