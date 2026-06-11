import { LazyStore } from "@tauri-apps/plugin-store";

export type ThemeMode = "system" | "light" | "dark";
export type Accent =
  | "indigo"
  | "violet"
  | "emerald"
  | "amber"
  | "rose"
  | "cyan"
  | "slate";

export const STORE_FILE = "settings.json";

let storeInstance: LazyStore | null = null;
function store(): LazyStore {
  if (!storeInstance) {
    storeInstance = new LazyStore(STORE_FILE);
  }
  return storeInstance;
}

export async function getTheme(): Promise<ThemeMode> {
  return ((await store().get<ThemeMode>("theme")) ?? "system") as ThemeMode;
}

export async function setTheme(mode: ThemeMode): Promise<void> {
  await store().set("theme", mode);
  await store().save();
  applyTheme(mode);
}

export async function getAccent(): Promise<Accent> {
  return ((await store().get<Accent>("accent")) ?? "violet") as Accent;
}

export async function setAccent(accent: Accent): Promise<void> {
  await store().set("accent", accent);
  await store().save();
  applyAccent(accent);
}

export function applyTheme(mode: ThemeMode): void {
  const root = document.documentElement;
  const prefersDark =
    typeof window !== "undefined"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
      : false;
  const dark = mode === "dark" || (mode === "system" && prefersDark);
  root.classList.toggle("dark", dark);
  root.dataset.themeMode = mode;
}

export function applyAccent(accent: Accent): void {
  document.documentElement.dataset.accent = accent;
}

export async function applyStoredTheme(): Promise<void> {
  const [mode, accent] = await Promise.all([getTheme(), getAccent()]);
  applyTheme(mode);
  applyAccent(accent);

  // React to system theme changes when in `system` mode.
  if (typeof window !== "undefined") {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    mql.addEventListener("change", async () => {
      const m = await getTheme();
      if (m === "system") applyTheme("system");
    });
  }
}
