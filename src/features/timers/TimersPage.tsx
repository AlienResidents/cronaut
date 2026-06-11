import { useEffect, useState } from "react";
import {
  listTimers,
  enableTimer,
  disableTimer,
  removeTimer,
  runTimerNow,
  type Timer,
} from "@/lib/api";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/Card";
import { Switch } from "@/components/ui/Switch";
import { formatDate } from "@/lib/utils";
import { Plus, Play, Trash2, RefreshCw, AlertTriangle } from "lucide-react";

export function TimersPage() {
  const [timers, setTimers] = useState<Timer[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      setTimers(await listTimers());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    refresh();
  }, []);

  async function onToggle(t: Timer, enabled: boolean) {
    try {
      if (enabled) await enableTimer(t.name);
      else await disableTimer(t.name);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function onRunNow(t: Timer) {
    try {
      await runTimerNow(t.name);
    } catch (e) {
      setError(String(e));
    }
  }

  async function onRemove(t: Timer) {
    if (
      !confirm(
        `Remove timer "${t.name}"?\n\nBoth ${t.name}.timer and ${t.name}.service will be deleted.`,
      )
    )
      return;
    try {
      await removeTimer(t.name);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  return (
    <div className="flex flex-col gap-6 p-8">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Systemd Timers</h1>
          <p className="text-sm text-[var(--text-muted)] mt-1">
            Manage{" "}
            <code className="text-xs bg-[var(--surface-muted)] px-1.5 py-0.5 rounded">
              systemctl --user
            </code>{" "}
            timers.
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="icon"
            onClick={refresh}
            disabled={loading}
            aria-label="Refresh"
          >
            <RefreshCw className={loading ? "animate-spin h-4 w-4" : "h-4 w-4"} />
          </Button>
          <Button variant="primary" disabled>
            <Plus className="h-4 w-4" /> New timer
          </Button>
        </div>
      </header>

      {error && (
        <Card className="border-[var(--danger)]">
          <CardContent className="flex items-start gap-3 text-sm">
            <AlertTriangle className="h-5 w-5 shrink-0 text-[var(--danger)]" />
            <pre className="whitespace-pre-wrap font-mono text-xs">{error}</pre>
          </CardContent>
        </Card>
      )}

      {loading && timers.length === 0 ? (
        <p className="text-sm text-[var(--text-muted)]">Loading…</p>
      ) : timers.length === 0 ? (
        <Card>
          <CardContent className="text-sm text-[var(--text-muted)]">
            No <code>--user</code> timers found. Click <em>New timer</em> to create one.
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-3">
          {timers.map((t) => (
            <Card key={t.name}>
              <CardHeader>
                <div className="flex items-start justify-between gap-4">
                  <div className="flex flex-col gap-0.5 min-w-0">
                    <CardTitle className="flex items-center gap-2">
                      <span className="truncate">{t.name}</span>
                      {t.active && (
                        <span className="text-xs px-2 py-0.5 rounded-full bg-[var(--success)]/10 text-[var(--success)] border border-[var(--success)]/20">
                          active
                        </span>
                      )}
                      {!t.managed && (
                        <span className="text-xs px-2 py-0.5 rounded-full bg-[var(--warning)]/10 text-[var(--warning)] border border-[var(--warning)]/20">
                          unmanaged
                        </span>
                      )}
                    </CardTitle>
                    <CardDescription className="truncate">{t.description || "—"}</CardDescription>
                  </div>
                  <div className="flex items-center gap-2 shrink-0">
                    <Switch
                      checked={t.enabled}
                      onCheckedChange={(v) => onToggle(t, v)}
                      aria-label={`Toggle ${t.name}`}
                    />
                  </div>
                </div>
              </CardHeader>
              <CardContent className="grid gap-2 text-sm">
                <Detail label="Schedule" value={t.schedule || "—"} mono />
                <Detail label="Command" value={t.command || "—"} mono />
                <Detail label="Next" value={formatDate(t.next_trigger)} />
                <Detail label="Last" value={formatDate(t.last_trigger)} />
                <div className="flex justify-end gap-2 pt-2">
                  <Button variant="ghost" size="sm" onClick={() => onRunNow(t)}>
                    <Play className="h-3.5 w-3.5" /> Run now
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => onRemove(t)}
                    disabled={!t.managed}
                    title={t.managed ? "Remove timer" : "Cannot remove unmanaged units"}
                  >
                    <Trash2 className="h-3.5 w-3.5" /> Remove
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}

function Detail({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="grid grid-cols-[5rem_1fr] gap-3">
      <span className="text-[var(--text-faint)] uppercase tracking-wider text-xs pt-0.5">
        {label}
      </span>
      <span className={mono ? "font-mono text-xs break-all" : "text-[var(--text-primary)]"}>
        {value}
      </span>
    </div>
  );
}
