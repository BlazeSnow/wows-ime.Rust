import { useState } from "react";
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
import { Button } from "../components/ui/button";
import { Separator } from "../components/ui/separator";
import {
  type InputMethodDefinition,
  existingConfigCount,
  hasGameExecutable,
  resolveTargetConfigFiles,
  writeConfigFiles,
} from "../lib/api";
import { describeError } from "../lib/errors";
import { AddInputMethodDialog } from "./home/AddInputMethodDialog";
import { ConfirmStep } from "./home/ConfirmStep";
import {
  DeleteGamePathDialog,
  DeleteInputMethodDialog,
} from "./home/DeleteDialogs";
import { GamePathStep } from "./home/GamePathStep";
import { InputMethodStep } from "./home/InputMethodStep";
import { TOTAL_STEPS } from "./home/constants";
import { useGamePaths } from "./home/useGamePaths";
import { useInputMethods } from "./home/useInputMethods";

/** 写入前的确认信息：已有多少个配置文件、将新增多少个。 */
interface WriteConfirm {
  open: boolean;
  existing: number;
  total: number;
}

/** 主页面：游戏路径 → 输入法选择 → 确认写入 三步向导。 */
export function HomePage() {
  const { t } = useTranslation();
  const [step, setStep] = useState(1);
  const [addImeOpen, setAddImeOpen] = useState(false);
  const [writeConfirm, setWriteConfirm] = useState<WriteConfirm>({
    open: false,
    existing: 0,
    total: 0,
  });

  const gamePaths = useGamePaths();
  const inputMethods = useInputMethods();

  const goNext = async () => {
    if (step === 1 && gamePaths.selectedGamePath.length === 0) {
      toast.warning(t("Status/GameRootInvalid"));
      return;
    }
    if (step === 2 && inputMethods.selectedInputMethods.length === 0) {
      toast.warning(t("Status/SelectAtLeastOneIme"));
      return;
    }
    if (step === 1) {
      // 输入法列表在第 2 步才可见，扫描结果到这一步再提示
      await inputMethods.reportScanOutcome();
    }
    if (step < TOTAL_STEPS) {
      setStep(step + 1);
    }
  };

  const prepareWrite = async () => {
    if (inputMethods.selectedInputMethods.length === 0) {
      toast.warning(t("Status/SelectAtLeastOneIme"));
      return;
    }
    if (gamePaths.selectedGamePath.length === 0) {
      toast.error(t("Status/GameRootInvalid"));
      return;
    }

    const hasExe = await hasGameExecutable(gamePaths.selectedGamePath).catch(
      () => false,
    );
    if (!hasExe) {
      toast.error(t("Status/GameExeNotFound"));
      return;
    }

    const targets = await resolveTargetConfigFiles(
      gamePaths.selectedGamePath,
    ).catch(() => [] as string[]);
    if (targets.length === 0) {
      toast.error(t("Status/BinVersionDirectoryNotFound"));
      return;
    }

    const existing = await existingConfigCount(gamePaths.selectedGamePath).catch(
      () => 0,
    );
    setWriteConfirm({ open: true, existing, total: targets.length });
  };

  const confirmWrite = async () => {
    setWriteConfirm((current) => ({ ...current, open: false }));
    const definitions: InputMethodDefinition[] =
      inputMethods.selectedInputMethods.map((item) => ({
        displayName: item.displayName,
        category: item.category,
      }));
    try {
      const written = await writeConfigFiles(
        gamePaths.selectedGamePath,
        definitions,
      );
      toast.success(t("Status/WriteSucceededWithCount", { 0: written }));
      gamePaths.persistSelection(gamePaths.selectedGamePath);
      inputMethods.persistCustomInputMethods(inputMethods.inputMethods);
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
        <GamePathStep
          gamePaths={gamePaths.gamePaths}
          selectedGamePath={gamePaths.selectedGamePath}
          onSelect={gamePaths.selectGamePath}
          onAdd={() => void gamePaths.addCustomGamePath()}
          onOpenFolder={() => void gamePaths.openGameFolder()}
          onRequestDeletion={gamePaths.requestDeletion}
        />
      )}

      {step === 2 && (
        <InputMethodStep
          inputMethods={inputMethods.inputMethods}
          allSelected={inputMethods.allSelected}
          selectedCount={inputMethods.selectedCount}
          onToggle={inputMethods.toggleInputMethod}
          onSelectAll={inputMethods.toggleSelectAll}
          onChangeCategory={inputMethods.changeCategory}
          onAdd={() => setAddImeOpen(true)}
          onRescan={inputMethods.rescan}
          onRequestDeletion={inputMethods.requestDeletion}
        />
      )}

      {step === TOTAL_STEPS && (
        <ConfirmStep
          gameRoot={gamePaths.selectedGamePath}
          inputMethods={inputMethods.selectedInputMethods}
        />
      )}

      <DeleteGamePathDialog
        option={gamePaths.pendingDeletion}
        onCancel={gamePaths.cancelDeletion}
        onConfirm={gamePaths.confirmDeletion}
      />

      <DeleteInputMethodDialog
        item={inputMethods.pendingDeletion}
        onCancel={inputMethods.cancelDeletion}
        onConfirm={inputMethods.confirmDeletion}
      />

      <AddInputMethodDialog
        open={addImeOpen}
        onOpenChange={setAddImeOpen}
        onConfirm={inputMethods.addCustomInputMethod}
      />

      <WriteConfirmDialog
        confirm={writeConfirm}
        onOpenChange={(open) =>
          setWriteConfirm((current) => ({ ...current, open }))
        }
        onConfirm={() => void confirmWrite()}
      />
    </div>
  );
}

interface WriteConfirmDialogProps {
  confirm: WriteConfirm;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
}

/** 写入前确认：已有配置文件时提示覆盖，否则提示新增。 */
function WriteConfirmDialog({
  confirm,
  onOpenChange,
  onConfirm,
}: WriteConfirmDialogProps) {
  const { t } = useTranslation();
  const overwrite = confirm.existing > 0;

  return (
    <AlertDialog open={confirm.open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            {overwrite
              ? t("Dialog/Overwrite/Title")
              : t("Dialog/AddConfig/Title")}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {overwrite
              ? t("Dialog/Overwrite/Content", { 0: confirm.existing })
              : t("Dialog/AddConfig/Content", { 0: confirm.total })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t("Dialog/Common/Cancel")}</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm}>
            {overwrite
              ? t("Dialog/Overwrite/PrimaryButton")
              : t("Dialog/AddConfig/PrimaryButton")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
