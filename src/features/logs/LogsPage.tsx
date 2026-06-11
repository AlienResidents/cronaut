import { useEffect, useRef, useState } from "react";
import {
  listTimers,
  fetchLogs,
  startTail,
  stopTail,
  subscribeTail,
  type LogSource,
  type Timer,
} from "@/lib/api";
import { Button } from "@/components/ui/Button";
import { Card, CardContent } from "@/components/ui/Card";
import { Switch } from "@/components/ui/Switch";
import { Label } from "@/components/ui/Input";
import { RefreshCw, AlertTriangle } from "lucide-react";

type SourceKey = "user_cron" | string; // string = unit name

export function LogsPage() {
  const [timers, setTimers] = useState<Timer[]>([]);
  const [source, setSource] = useState<SourceKey>("user_cron");
  const [lines, setLines] = useState<string[]>([]);
  const [following, setFollowing] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const tailIdRef = useRef<string | null>(null);
  const unsubRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    listTimers().then(setTimers).catch((e) => setError(String(e)));
  }, []);

  function buildSource(): LogSource {
    if (source === "user_cron") return { kind: "user_cron" };
    return { kind: "unit", unit: `${source}.service` };
  }

  async function snapshot() {
    setLoading(true);
    setError(null);
    try {
      const result = await fetchLogs({ source: buildSource(), lines: 500 });
      setLines(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function startFollow() {
    if (following) return;
    const id = `tail-${Date.now()}`;
    tailIdRef.current = id;
    setLines([]);
    setError(null);
    try {
      const unsub = await subscribeTail(id, (line) => {
        setLines((prev) => [...prev.slice(-2999), line]);
      });
      unsubRef.current = unsub;
      await startTail({ id, source: buildSource(), lines: 200 });
      setFollowing(true);
    } catch (e) {
      setError(String(e));
      unsubRef.current?.();
      unsubRef.current = null;
      tailIdRef.current = null;
    }
  }

  async function stopFollow() {
    if (!following) return;
    const id = tailIdRef.current;
    setFollowing(false);
    unsubRef.current?.();
    unsubRef.current = null;
    if (id) {
      try {
        await stopTail(id);
      } catch (e) {
        setError(String(e));
      }
    }
    tailIdRef.current = null;
  }

  useEffect(() => {
    return () => {
      if (tailIdRef.current) {
        stopTail(tailIdRef.current).catch(() => {});
        unsubRef.current?.();
      }
    };
  }, []);

  return (
    <div className="flex h-full flex-col gap-4 p-8">
      <header className="flex items-end justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Logs</h1>
          <p className="text-sm text-[var(--text-muted)] mt-1">
            Live or snapshot view from <code className="text-xs bg-[var(--surface-muted)] px-1.5 py-0.5 rounded">journalctl --user</code>.
          </p>
        </div>
      </header>

      <div className="flex flex-wrap items-end gap-4">
        <div className="flex flex-col gap-1.5 min-w-[280px]">
          <Label htmlFor="src">Source</Label>
          <select
            id="src"
            className="h-9 rounded-md border border-[var(--surface-border)] bg-[var(--surface-card)] px-2 text-sm"
            value={source}
            onChange={(e) => setSource(e.target.value)}
            disabled={following}
          >
            <option value="user_cron">User cron (CRON syslog tag)</option>
            {timers.map((t) => (
              <option key={t.name} value={t.name}>
                {t.name}.service
              </option>
            ))}
          </select>
        </div>
        <div className="flex items-center gap-2">
          <Switch checked={following} onCheckedChange={(v) => (v ? startFollow() : stopFollow())} />
          <span className="text-sm">Follow</span>
        </div>
        <Button variant="secondary" onClick={snapshot} disabled={following || loading}>
          <RefreshCw className={loading ? "animate-spin h-4 w-4" : "h-4 w-4"} />
          Snapshot last 500
        </Button>
      </div>

      {error && (
        <Card className="border-[var(--danger)]">
          <CardContent className="flex items-start gap-3 text-sm">
            <AlertTriangle className="h-5 w-5 shrink-0 text-[var(--danger)]" />
            <pre className="whitespace-pre-wrap font-mono text-xs">{error}</pre>
          </CardContent>
        </Card>
      )}

      <Card className="flex-1 min-h-0 overflow-hidden">
        <CardContent className="p-0 h-full">
          <pre className="h-full overflow-auto p-4 text-xs leading-relaxed font-mono whitespace-pre-wrap text-[var(--text-primary)]">
{lines.length === 0
              ? "(no log lines yet — pick a source and click Snapshot, or toggle Follow)"
              : lines.join("\n")}
          </pre>
        </CardContent>
      </Card>
    </div>
  );
}
