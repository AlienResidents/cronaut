import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ---- Desktop env ----------------------------------------------------------

export type DesktopEnv =
  | "xfce"
  | "kde"
  | "gnome"
  | "cinnamon"
  | "mate"
  | "lxde"
  | "lxqt"
  | "pantheon"
  | "budgie"
  | "unity"
  | "deepin"
  | "unknown";

export interface DesktopReport {
  env: DesktopEnv;
  native_tray: boolean;
  raw_xdg_current_desktop: string;
  session_type: string;
}

export const desktopReport = () => invoke<DesktopReport>("desktop_report");

// ---- Timers ---------------------------------------------------------------

export interface Timer {
  name: string;
  description: string;
  schedule: string;
  command: string;
  working_directory: string | null;
  enabled: boolean;
  active: boolean;
  last_trigger: string | null;
  next_trigger: string | null;
  managed: boolean;
}

export interface TimerSpec {
  name: string;
  description: string;
  schedule: string;
  command: string;
  working_directory?: string | null;
  enable?: boolean;
}

export const listTimers = () => invoke<Timer[]>("list_timers");
export const createTimer = (spec: TimerSpec) => invoke<void>("create_timer", { spec });
export const updateTimer = (spec: TimerSpec) => invoke<void>("update_timer", { spec });
export const enableTimer = (name: string) => invoke<void>("enable_timer", { name });
export const disableTimer = (name: string) => invoke<void>("disable_timer", { name });
export const runTimerNow = (name: string) => invoke<void>("run_timer_now", { name });
export const removeTimer = (name: string) => invoke<void>("remove_timer", { name });

// ---- Crontab --------------------------------------------------------------

export type CronLine =
  | {
      kind: "entry";
      id: string;
      line_number: number;
      schedule: string;
      command: string;
      disabled: boolean;
      raw: string;
    }
  | {
      kind: "passthrough";
      id: string;
      line_number: number;
      raw: string;
    };

export interface CronEntrySpec {
  schedule: string;
  command: string;
}

export const listCrontab = () => invoke<CronLine[]>("list_crontab");
export const addCrontabEntry = (spec: CronEntrySpec) =>
  invoke<void>("add_crontab_entry", { spec });
export const updateCrontabEntry = (lineNumber: number, spec: CronEntrySpec) =>
  invoke<void>("update_crontab_entry", { lineNumber, spec });
export const toggleCrontabEntry = (lineNumber: number, enabled: boolean) =>
  invoke<void>("toggle_crontab_entry", { lineNumber, enabled });
export const removeCrontabEntry = (lineNumber: number) =>
  invoke<void>("remove_crontab_entry", { lineNumber });
export const validateCronExpression = (expr: string) =>
  invoke<void>("validate_cron_expression", { expr });

// ---- Logs -----------------------------------------------------------------

export type LogSource =
  | { kind: "unit"; unit: string }
  | { kind: "user_cron" };

export interface FetchSpec {
  source: LogSource;
  lines: number;
}

export interface TailSpec {
  id: string;
  source: LogSource;
  lines: number;
}

export const fetchLogs = (spec: FetchSpec) => invoke<string[]>("fetch_logs", { spec });
export const startTail = (spec: TailSpec) => invoke<void>("start_tail", { spec });
export const stopTail = (id: string) => invoke<void>("stop_tail", { id });

export async function subscribeTail(
  id: string,
  onLine: (line: string) => void,
  onEnd?: () => void,
): Promise<UnlistenFn> {
  const unListen = await listen<string>(`log:line:${id}`, (event) => onLine(event.payload));
  const unEnd = await listen<void>(`log:end:${id}`, () => onEnd?.());
  return () => {
    unListen();
    unEnd();
  };
}
