import { useTranslation } from "react-i18next";

import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "../../components/ui/card";
import { Checkbox } from "../../components/ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../components/ui/select";
import { type ImeCategory } from "../../lib/api";
import { CATEGORY_KEYS } from "./constants";
import type { ImeItem } from "./types";

interface InputMethodStepProps {
  inputMethods: ImeItem[];
  /** 是否全部勾选。 */
  allSelected: boolean;
  /** 已勾选数量（用于半选态）。 */
  selectedCount: number;
  onToggle: (displayName: string, checked: boolean) => void;
  onSelectAll: (checked: boolean) => void;
  onChangeCategory: (displayName: string, category: ImeCategory) => void;
  onAdd: () => void;
  onRescan: () => void;
  onRequestDeletion: (item: ImeItem) => void;
}

/** 第 2 步：勾选要写入的输入法并调整分类。 */
export function InputMethodStep({
  inputMethods,
  allSelected,
  selectedCount,
  onToggle,
  onSelectAll,
  onChangeCategory,
  onAdd,
  onRescan,
  onRequestDeletion,
}: InputMethodStepProps) {
  const { t } = useTranslation();
  // 部分勾选时表头复选框显示半选态
  const selectAllState: boolean | "indeterminate" = allSelected
    ? true
    : selectedCount > 0
      ? "indeterminate"
      : false;

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("ImeSectionTitle.Text")}</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <div className="grid grid-cols-[1fr_10rem_6rem] items-center gap-2 text-xs text-muted-foreground">
          <div className="flex items-center gap-2">
            <Checkbox
              id="ime-select-all"
              aria-label={t("Ime/SelectAll")}
              title={t("Ime/SelectAll")}
              checked={selectAllState}
              onCheckedChange={(checked) => onSelectAll(checked === true)}
            />
            <span>{t("ImeTableHeaderName.Text")}</span>
          </div>
          <span>{t("ImeTableHeaderCategory.Text")}</span>
          <span>{t("ImeTableHeaderAction.Text")}</span>
        </div>
        <div className="flex max-h-96 flex-col gap-1 overflow-y-auto">
          {inputMethods.map((item) => (
            <div
              key={item.displayName}
              className="grid grid-cols-[1fr_10rem_6rem] items-center gap-2 rounded-lg border px-3 py-1.5 text-sm"
            >
              <div className="flex min-w-0 items-center gap-2">
                <label className="flex min-w-0 items-center gap-2">
                  <Checkbox
                    aria-label={item.displayName}
                    checked={item.isSelected}
                    onCheckedChange={(checked) =>
                      onToggle(item.displayName, checked === true)
                    }
                  />
                  <span className="truncate">{item.displayName}</span>
                </label>
                {!item.isCustom && (
                  <Badge variant="secondary" className="shrink-0">
                    {t("Ime/SystemSource")}
                  </Badge>
                )}
              </div>
              <Select
                value={item.category}
                onValueChange={(value) =>
                  onChangeCategory(item.displayName, value as ImeCategory)
                }
              >
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {(Object.keys(CATEGORY_KEYS) as ImeCategory[]).map(
                    (category) => (
                      <SelectItem key={category} value={category}>
                        {t(CATEGORY_KEYS[category])}
                      </SelectItem>
                    ),
                  )}
                </SelectContent>
              </Select>
              <div className="flex justify-end">
                {item.isCustom && (
                  <Button
                    variant="ghost"
                    size="xs"
                    onClick={() => onRequestDeletion(item)}
                  >
                    {t("DeleteCustomImeButtonTemplate.Content")}
                  </Button>
                )}
              </div>
            </div>
          ))}
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={onAdd}>
            {t("AddCustomImeButton.Content")}
          </Button>
          <Button variant="outline" onClick={onRescan}>
            {t("RefreshImeButton.Content")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
