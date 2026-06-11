import { useEffect, useState } from "react";
import { Sidebar } from "@/components/Sidebar";
import { TimersPage } from "@/features/timers/TimersPage";
import { CrontabPage } from "@/features/crontab/CrontabPage";
import { LogsPage } from "@/features/logs/LogsPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { AboutPage } from "@/features/about/AboutPage";
import { applyStoredTheme } from "@/lib/theme";

export type Page = "timers" | "crontab" | "logs" | "settings" | "about";

export default function App() {
  const [page, setPage] = useState<Page>("timers");

  useEffect(() => {
    // re-apply in case main.tsx's pre-mount call failed
    applyStoredTheme().catch(() => {});
  }, []);

  return (
    <div className="flex h-full w-full">
      <Sidebar current={page} onNavigate={setPage} />
      <main className="flex-1 overflow-auto">
        {page === "timers" && <TimersPage />}
        {page === "crontab" && <CrontabPage />}
        {page === "logs" && <LogsPage />}
        {page === "settings" && <SettingsPage />}
        {page === "about" && <AboutPage />}
      </main>
    </div>
  );
}
