import { useTranslation } from "react-i18next";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../../components/ui/card";
import { Separator } from "../../components/ui/separator";
import { CATEGORY_KEYS } from "./constants";
import type { ImeItem } from "./types";

interface ConfirmStepProps {
  gameRoot: string;
  inputMethods: ImeItem[];
}

/** 第 3 步：确认游戏根目录与已选输入法。 */
export function ConfirmStep({ gameRoot, inputMethods }: ConfirmStepProps) {
  const { t } = useTranslation();

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("WriteConfigCardHeader.Text")}</CardTitle>
        <CardDescription>{t("GameRootLabel.Text")}</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <p className="text-sm">{gameRoot}</p>
        <Separator />
        <p className="text-sm font-medium">{t("ConfirmImeSectionHeader.Text")}</p>
        <ul className="flex flex-col gap-1 text-sm text-muted-foreground">
          {inputMethods.map((item, index) => (
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
  );
}
