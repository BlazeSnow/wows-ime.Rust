import { useTranslation } from "react-i18next";

import { Button } from "../../components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../../components/ui/card";
import type { GamePathOption } from "./types";

interface GamePathStepProps {
  gamePaths: GamePathOption[];
  selectedGamePath: string;
  onSelect: (path: string) => void;
  onAdd: () => void;
  onOpenFolder: () => void;
  onRequestDeletion: (option: GamePathOption) => void;
}

/** 第 1 步：选择游戏根目录（预设路径或自定义路径）。 */
export function GamePathStep({
  gamePaths,
  selectedGamePath,
  onSelect,
  onAdd,
  onOpenFolder,
  onRequestDeletion,
}: GamePathStepProps) {
  const { t } = useTranslation();

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t("GameRootLabel.Text")}</CardTitle>
        <CardDescription>{t("GamePathSectionHint.Text")}</CardDescription>
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
                onChange={() => onSelect(option.path)}
              />
              <span className="font-medium">{option.displayName}</span>
              <span className="min-w-0 flex-1 truncate text-muted-foreground">
                {option.path}
              </span>
              {option.isCustom && (
                <Button
                  variant="ghost"
                  size="xs"
                  onClick={() => onRequestDeletion(option)}
                >
                  {t("DeleteCustomGamePathButtonTemplate.Content")}
                </Button>
              )}
            </label>
          ))}
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={onAdd}>
            {t("AddCustomGamePathButton.Content")}
          </Button>
          <Button
            variant="outline"
            disabled={selectedGamePath.length === 0}
            onClick={onOpenFolder}
          >
            {t("OpenFolderButton.Content")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
