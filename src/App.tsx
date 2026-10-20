import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Toaster } from "./components/ui/sonner";
import { cn } from "./lib/utils";
import { HomePage } from "./pages/HomePage";
import { SettingsPage } from "./pages/SettingsPage";

type PageKey = "home" | "settings";

const NAV_ITEMS: { key: PageKey; label: string }[] = [
  { key: "home", label: "Nav/Home" },
  { key: "settings", label: "Nav/Settings" },
];

export default function App() {
  const { t } = useTranslation();
  const [page, setPage] = useState<PageKey>("home");

  return (
    <div className="flex h-full flex-col bg-background text-foreground">
      <header className="flex items-baseline gap-3 border-b px-5 py-3">
        <span className="font-heading text-base font-medium">
          {t("App/Title")}
        </span>
        <span className="text-sm text-muted-foreground">
          {t("AppDescription")}
        </span>
      </header>
      <div className="flex min-h-0 flex-1">
        <nav className="flex w-40 shrink-0 flex-col gap-1 border-r p-3">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.key}
              type="button"
              onClick={() => setPage(item.key)}
              className={cn(
                "rounded-lg px-3 py-1.5 text-left text-sm transition-colors",
                page === item.key
                  ? "bg-secondary font-medium text-secondary-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground",
              )}
            >
              {t(item.label)}
            </button>
          ))}
        </nav>
        <main className="min-w-0 flex-1 overflow-y-auto p-5">
          {page === "home" ? <HomePage /> : <SettingsPage />}
        </main>
      </div>
      <Toaster />
    </div>
  );
}
