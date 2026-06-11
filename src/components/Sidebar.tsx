import { Clock, Calendar, ScrollText, Settings, Info, Rocket } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Page } from "@/App";

interface NavItem {
  id: Page;
  label: string;
  icon: React.ElementType;
}

const NAV: NavItem[] = [
  { id: "timers", label: "Timers", icon: Clock },
  { id: "crontab", label: "Crontab", icon: Calendar },
  { id: "logs", label: "Logs", icon: ScrollText },
  { id: "settings", label: "Settings", icon: Settings },
  { id: "about", label: "About", icon: Info },
];

interface SidebarProps {
  current: Page;
  onNavigate: (page: Page) => void;
}

export function Sidebar({ current, onNavigate }: SidebarProps) {
  return (
    <aside className="flex h-full w-56 flex-col border-r border-[var(--surface-border)] bg-[var(--surface-card)]">
      <div className="flex items-center gap-2 px-4 py-5">
        <div className="grid h-9 w-9 place-items-center rounded-md bg-[var(--accent)] text-[var(--accent-fg)]">
          <Rocket className="h-5 w-5" />
        </div>
        <div className="flex flex-col leading-tight">
          <span className="text-sm font-semibold tracking-tight">Cronaut</span>
          <span className="text-xs text-[var(--text-muted)]">v0.1.0</span>
        </div>
      </div>
      <nav className="flex flex-col gap-1 px-2 py-2">
        {NAV.map(({ id, label, icon: Icon }) => {
          const active = current === id;
          return (
            <button
              key={id}
              type="button"
              onClick={() => onNavigate(id)}
              className={cn(
                "group flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors cursor-pointer",
                active
                  ? "bg-[var(--accent-soft)] text-[var(--accent)] font-medium"
                  : "text-[var(--text-muted)] hover:bg-[var(--surface-muted)] hover:text-[var(--text-primary)]",
              )}
            >
              <Icon className="h-4 w-4" />
              <span>{label}</span>
            </button>
          );
        })}
      </nav>
      <div className="mt-auto px-4 py-4 text-xs text-[var(--text-faint)]">
        Local-only • No telemetry
      </div>
    </aside>
  );
}
