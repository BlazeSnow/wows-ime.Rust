import { useTranslation } from "react-i18next";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../components/ui/card";

/** 主页面：三步向导（游戏路径 → 输入法选择 → 确认写入）。M3 填充具体交互。 */
export function HomePage() {
  const { t } = useTranslation();

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("Nav.Home")}</CardTitle>
        <CardDescription>{t("App.Subtitle")}</CardDescription>
      </CardHeader>
      <CardContent className="text-sm text-muted-foreground">
        {t("Home.ComingSoon")}
      </CardContent>
    </Card>
  );
}
