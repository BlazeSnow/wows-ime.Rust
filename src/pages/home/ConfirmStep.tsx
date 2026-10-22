import { useTranslation } from "react-i18next";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../../components/ui/card";
import { Separator } from "../../components/ui/separator";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../../components/ui/table";
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
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-12" />
              <TableHead>{t("ImeTableHeaderName.Text")}</TableHead>
              <TableHead className="w-32">
                {t("ImeTableHeaderCategory.Text")}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {inputMethods.map((item, index) => (
              <TableRow key={item.displayName}>
                <TableCell className="text-muted-foreground">
                  {index + 1}
                </TableCell>
                <TableCell className="font-medium">
                  {item.displayName}
                </TableCell>
                <TableCell>{t(CATEGORY_KEYS[item.category])}</TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}
