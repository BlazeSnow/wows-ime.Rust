import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../../components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../../components/ui/dialog";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import { type ImeCategory } from "../../lib/api";
import { CATEGORY_KEYS } from "./constants";

interface AddInputMethodDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: (displayName: string, category: ImeCategory) => void;
}

/** 添加自定义输入法：名称 + 分类。 */
export function AddInputMethodDialog({
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
          <Label htmlFor="custom-ime-name">
            {t("Dialog/AddCustomIme/NameLabel")}
          </Label>
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
          <Button variant="outline" onClick={() => onOpenChange(false)}>
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
