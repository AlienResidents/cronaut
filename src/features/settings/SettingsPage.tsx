import { useEffect, useState } from "react";
import { LazyStore } from "@tauri-apps/plugin-store";
import { desktopReport, type DesktopReport } from "@/lib/api";
import {
  setTheme,
  setAccent,
  type ThemeMode,
  type Accent,
  STORE_FILE,
} from "@/lib/theme";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/Card";
import { Switch } from "@/components/ui/Switch";
import { Label } from "@/components/ui/Input";
import { cn } from "@/lib/utils";
import { AlertTriangle } from "lucide-react";

type CloseAction = "minimize_to_tray" | "minimize_to_taskbar" | "quit";

const ACCENTS: { id: Accent; label: string; swatch: string }[] = [
  { id: "violet", label: "Violet", swatch: "oklch(0.62 0.21 295)" },
  { id: "indigo", label: "Indigo", swatch: "oklch(0.6 0.2 270)" },
  { id: "emerald", label: "Emerald", swatch: "oklch(0.65 0.18 155)" },
  { id: "amber", label: "Amber", swatch: "oklch(0.78 0.17 80)" },
  { id: "rose", label: "Rose", swatch: "oklch(0.66 0.22 15)" },
  { id: "cyan", label: "Cyan", swatch: "oklch(0.7 0.15 215)" },
  { id: "slate", label: "Slate", swatch: "oklch(0.5 0.04 270)" },
];

export function SettingsPage() {
  const [theme, setThemeState] = useState<ThemeMode>("system");
  const [accent, setAccentState] = useState<Accent>("violet");
  const [closeAction, setCloseAction] = useState<CloseAction>("minimize_to_tray");
  const [startMinimized, setStartMinimized] = useState(false);
  const [notify, setNotify] = useState(false);
  const [report, setReport] = useState<DesktopReport | null>(null);
  const [store, setStore] = useState<LazyStore | null>(null);

  useEffect(() => {
    const s = new LazyStore(STORE_FILE);
    setStore(s);
    (async () => {
      setThemeState(((await s.get<ThemeMode>("theme")) ?? "system") as ThemeMode);
      setAccentState(((await s.get<Accent>("accent")) ?? "violet") as Accent);
      setCloseAction(
        ((await s.get<CloseAction>("close_action")) ?? "minimize_to_tray") as CloseAction,
      );
      setStartMinimized(((await s.get<boolean>("start_minimized")) ?? false) as boolean);
      setNotify(((await s.get<boolean>("notify_on_run")) ?? false) as boolean);
    })();
    desktopReport().then(setReport).catch(() => {});
  }, []);

  async function persist<T>(key: string, value: T) {
    if (!store) return;
    await store.set(key, value);
    await store.save();
  }

  return (
    <div className="flex flex-col gap-6 p-8 max-w-3xl">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
        <p className="text-sm text-[var(--text-muted)] mt-1">
          All settings are stored locally in <code className="text-xs bg-[var(--surface-muted)] px-1.5 py-0.5 rounded">~/.config/com.alienresidents.cronaut/{STORE_FILE}</code>.
        </p>
      </header>

      <Card>
        <CardHeader>
          <CardTitle>Appearance</CardTitle>
          <CardDescription>Theme mode and accent color.</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-6">
          <div className="grid gap-2">
            <Label>Theme</Label>
            <div className="flex gap-2">
              {(["system", "light", "dark"] as ThemeMode[]).map((m) => (
                <button
                  key={m}
                  type="button"
                  onClick={async () => {
                    setThemeState(m);
                    await setTheme(m);
                  }}
                  className={cn(
                    "px-3 py-1.5 rounded-md text-sm border capitalize cursor-pointer transition-colors",
                    theme === m
                      ? "bg-[var(--accent-soft)] text-[var(--accent)] border-[var(--accent)]"
                      : "bg-[var(--surface-card)] border-[var(--surface-border)] hover:bg-[var(--surface-muted)]",
                  )}
                >
                  {m}
                </button>
              ))}
            </div>
          </div>
          <div className="grid gap-2">
            <Label>Accent</Label>
            <div className="flex flex-wrap gap-2">
              {ACCENTS.map((a) => (
                <button
                  key={a.id}
                  type="button"
                  onClick={async () => {
                    setAccentState(a.id);
                    await setAccent(a.id);
                  }}
                  title={a.label}
                  aria-label={a.label}
                  className={cn(
                    "h-8 w-8 rounded-full border-2 cursor-pointer transition-transform",
                    accent === a.id
                      ? "border-[var(--text-primary)] scale-110"
                      : "border-[var(--surface-border)]",
                  )}
                  style={{ background: a.swatch }}
                />
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Window behaviour</CardTitle>
          <CardDescription>What happens when you close or launch Cronaut.</CardDescription>
        </CardHeader>
        <CardContent className="grid gap-6">
          <div className="grid gap-2">
            <Label>On window close</Label>
            <div className="flex flex-col gap-2">
              {(
                [
                  ["minimize_to_tray", "Minimise to system tray (default)"],
                  ["minimize_to_taskbar", "Minimise to taskbar"],
                  ["quit", "Quit the app"],
                ] as [CloseAction, string][]
              ).map(([id, label]) => (
                <label
                  key={id}
                  className="flex items-center gap-2 text-sm cursor-pointer"
                >
                  <input
                    type="radio"
                    name="closeAction"
                    checked={closeAction === id}
                    onChange={async () => {
                      setCloseAction(id);
                      await persist("close_action", id);
                    }}
                  />
                  {label}
                </label>
              ))}
            </div>
            {report && !report.native_tray && closeAction === "minimize_to_tray" && (
              <p className="flex items-start gap-2 text-xs text-[var(--warning)] mt-1">
                <AlertTriangle className="h-3.5 w-3.5 mt-0.5 shrink-0" />
                {report.env === "gnome"
                  ? "GNOME does not ship a system tray. Install the AppIndicator extension or switch to taskbar fallback."
                  : `Detected desktop "${report.env}" may not show tray icons. Switch to taskbar fallback if the icon is invisible.`}
              </p>
            )}
          </div>
          <div className="flex items-center justify-between">
            <div className="flex flex-col">
              <Label>Start minimised</Label>
              <span className="text-xs text-[var(--text-muted)]">
                Launch hidden; only show on tray click.
              </span>
            </div>
            <Switch
              checked={startMinimized}
              onCheckedChange={async (v) => {
                setStartMinimized(v);
                await persist("start_minimized", v);
              }}
            />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Notifications</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <div className="flex flex-col">
              <Label>Notify on managed run</Label>
              <span className="text-xs text-[var(--text-muted)]">
                Show a desktop notification when a managed timer fires.
              </span>
            </div>
            <Switch
              checked={notify}
              onCheckedChange={async (v) => {
                setNotify(v);
                await persist("notify_on_run", v);
              }}
            />
          </div>
        </CardContent>
      </Card>

      {report && (
        <Card>
          <CardHeader>
            <CardTitle>Diagnostics</CardTitle>
            <CardDescription>Read-only — useful for issue reports.</CardDescription>
          </CardHeader>
          <CardContent className="grid gap-1 font-mono text-xs">
            <div>desktop = {report.env}</div>
            <div>native_tray = {String(report.native_tray)}</div>
            <div>XDG_CURRENT_DESKTOP = {report.raw_xdg_current_desktop || "(unset)"}</div>
            <div>XDG_SESSION_TYPE = {report.session_type || "(unset)"}</div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
