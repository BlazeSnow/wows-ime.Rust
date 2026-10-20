import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import { Label } from "../components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../components/ui/select";
import { getAppVersion, setSetting } from "../lib/api";
import {
  LANGUAGE_AUTO,
  LANGUAGE_JA,
  LANGUAGE_PREFERENCES,
  LANGUAGE_ZH_HANS,
  LANGUAGE_ZH_HANT,
} from "../i18n";

const SETTING_LANGUAGE = "Settings.Language";

const LANGUAGE_LABELS: Record<string, string> = {
  [LANGUAGE_AUTO]: "Settings.LanguageAuto",
  [LANGUAGE_ZH_HANS]: "简体中文",
  [LANGUAGE_ZH_HANT]: "繁體中文",
  [LANGUAGE_JA]: "日本語",
};

/** 设置页：界面语言切换、版本信息与项目链接。 */
export function SettingsPage() {
  const { t, i18n } = useTranslation();
  const [version, setVersion] = useState("");
  const [preference, setPreference] = useState<string>(LANGUAGE_AUTO);

  useEffect(() => {
    void getAppVersion().then(setVersion).catch(() => setVersion(""));
  }, []);

  const changeLanguage = async (next: string) => {
    setPreference(next);
    await i18n.changeLanguage(next);
    await setSetting(SETTING_LANGUAGE, next).catch(() => undefined);
  };

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle>{t("Settings.Language")}</CardTitle>
          <CardDescription>{t("App.Subtitle")}</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-2">
          <Label htmlFor="language-select">{t("Settings.Language")}</Label>
          <Select value={preference} onValueChange={(value) => void changeLanguage(value)}>
            <SelectTrigger id="language-select" className="w-56">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LANGUAGE_PREFERENCES.map((code) => (
                <SelectItem key={code} value={code}>
                  {LANGUAGE_LABELS[code]?.startsWith("Settings.")
                    ? t(LANGUAGE_LABELS[code])
                    : LANGUAGE_LABELS[code]}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("Settings.Version")}</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          {version}
        </CardContent>
      </Card>
    </div>
  );
}
