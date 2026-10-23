import { useTranslation } from "react-i18next";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "../../components/ui/alert-dialog";
import type { GamePathOption, ImeItem } from "./types";

interface DeleteGamePathDialogProps {
  option: GamePathOption | null;
  onCancel: () => void;
  onConfirm: () => void;
}

/** 删除自定义游戏路径的确认框。 */
export function DeleteGamePathDialog({
  option,
  onCancel,
  onConfirm,
}: DeleteGamePathDialogProps) {
  const { t } = useTranslation();

  return (
    <AlertDialog
      open={option !== null}
      onOpenChange={(open) => {
        if (!open) {
          onCancel();
        }
      }}
    >
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            {t("Dialog/DeleteCustomGamePath/Title")}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {option
              ? t("Dialog/DeleteCustomGamePath/Content", {
                  0: option.displayName,
                  1: option.path,
                })
              : ""}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t("Dialog/Common/Cancel")}</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm}>
            {t("Dialog/DeleteCustomGamePath/PrimaryButton")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

interface DeleteInputMethodDialogProps {
  item: ImeItem | null;
  onCancel: () => void;
  onConfirm: () => void;
}

/** 删除自定义输入法的确认框。 */
export function DeleteInputMethodDialog({
  item,
  onCancel,
  onConfirm,
}: DeleteInputMethodDialogProps) {
  const { t } = useTranslation();

  return (
    <AlertDialog
      open={item !== null}
      onOpenChange={(open) => {
        if (!open) {
          onCancel();
        }
      }}
    >
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("Dialog/DeleteCustomIme/Title")}</AlertDialogTitle>
          <AlertDialogDescription>
            {item
              ? t("Dialog/DeleteCustomIme/Content", { 0: item.displayName })
              : ""}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t("Dialog/Common/Cancel")}</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm}>
            {t("Dialog/DeleteCustomIme/PrimaryButton")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
