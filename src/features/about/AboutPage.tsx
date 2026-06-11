import { openUrl } from "@tauri-apps/plugin-opener";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { ExternalLink } from "lucide-react";

export function AboutPage() {
  return (
    <div className="flex flex-col gap-6 p-8 max-w-3xl">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">About</h1>
      </header>

      <Card>
        <CardHeader>
          <CardTitle>Cronaut v0.1.0</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-4 text-sm">
          <p>
            A modern desktop UI for managing <code>systemd --user</code> timers and the current
            user's crontab on Debian-based Linux. Local-only, no telemetry, no network calls.
          </p>
          <ul className="grid gap-1 text-sm text-[var(--text-muted)]">
            <li>• systemd --user timer CRUD with safe ownership marker</li>
            <li>• User crontab CRUD with non-destructive disable</li>
            <li>• journalctl log snapshots and live tail</li>
            <li>• Themable (system/light/dark + 7 accents)</li>
            <li>• Tray with auto-detect; taskbar fallback on GNOME</li>
          </ul>
          <div className="flex flex-wrap gap-2 pt-2">
            <Button
              variant="secondary"
              onClick={() => openUrl("https://github.com/alienresidents/cronaut")}
            >
              <ExternalLink className="h-4 w-4" />
              Source on GitHub
            </Button>
            <Button
              variant="ghost"
              onClick={() => openUrl("https://github.com/alienresidents/cronaut/issues")}
            >
              <ExternalLink className="h-4 w-4" />
              Report an issue
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>License</CardTitle>
        </CardHeader>
        <CardContent className="text-sm">
          Released under the MIT License. See the <code>LICENSE</code> file in the source repo.
        </CardContent>
      </Card>
    </div>
  );
}
