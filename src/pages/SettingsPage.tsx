import { openPath } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";

import { Button } from "../components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../components/ui/card";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../components/ui/select";
import {
  LANGUAGE_AUTO,
  LANGUAGE_JA,
  LANGUAGE_PREFERENCES,
  LANGUAGE_ZH_HANS,
  LANGUAGE_ZH_HANT,
} from "../i18n";
import { getAppVersion, getLanguagePreference, setSetting } from "../lib/api";
import { PROJECT_REPOSITORY, PROJECT_WEBSITE } from "../lib/links";

const SETTING_LANGUAGE = "Settings.Language";

const LANGUAGE_LABELS: Record<string, string> = {
  [LANGUAGE_AUTO]: "Settings/Automatic",
  [LANGUAGE_ZH_HANS]: "简体中文",
  [LANGUAGE_ZH_HANT]: "繁體中文",
  [LANGUAGE_JA]: "日本語",
};

/** 设置页：界面语言切换、软件版本与项目链接。 */
export function SettingsPage() {
  const { t, i18n } = useTranslation();
  const [version, setVersion] = useState("");
  const [preference, setPreference] = useState<string>(LANGUAGE_AUTO);

  useEffect(() => {
    void getAppVersion().then(setVersion).catch(() => setVersion(""));
    void getLanguagePreference()
      .then(setPreference)
      .catch(() => setPreference(LANGUAGE_AUTO));
  }, []);

  const changeLanguage = async (next: string) => {
    setPreference(next);
    await i18n.changeLanguage(next);
    await setSetting(SETTING_LANGUAGE, next).catch(() => undefined);
  };

  const openLink = async (url: string, failureKey: string) => {
    const opened = await openPath(url).catch(() => false);
    if (!opened) {
      toast.error(t(failureKey));
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          {/* 卡片标题同时作为下拉框的可访问名称，不再另立一个同名 label */}
          <CardTitle id="language-setting-title">
            {t("Settings/LanguageLabel")}
          </CardTitle>
          <CardDescription>{t("Settings/LanguageHint")}</CardDescription>
        </CardHeader>
        <CardContent>
          <Select
            value={preference}
            onValueChange={(value) => void changeLanguage(value)}
          >
            <SelectTrigger
              id="language-select"
              aria-labelledby="language-setting-title"
              className="w-56"
            >
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {LANGUAGE_PREFERENCES.map((code) => (
                <SelectItem key={code} value={code}>
                  {LANGUAGE_LABELS[code].startsWith("Settings/")
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
          <CardTitle>{t("AppVersionCardHeader.Text")}</CardTitle>
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          {version}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("ProjectWebsiteCardHeader.Text")}</CardTitle>
          <CardDescription>{PROJECT_WEBSITE}</CardDescription>
        </CardHeader>
        <CardContent>
          <Button
            variant="outline"
            onClick={() =>
              void openLink(
                PROJECT_WEBSITE,
                "Status/OpenProjectWebsiteFailed",
              )
            }
          >
            {t("OpenProjectWebsiteButton.Content")}
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("ProjectRepositoryCardHeader.Text")}</CardTitle>
          <CardDescription>{PROJECT_REPOSITORY}</CardDescription>
        </CardHeader>
        <CardContent>
          <Button
            variant="outline"
            onClick={() =>
              void openLink(
                PROJECT_REPOSITORY,
                "Status/OpenProjectRepositoryFailed",
              )
            }
          >
            {t("OpenProjectRepositoryButton.Content")}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
