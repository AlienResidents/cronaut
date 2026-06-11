import { useEffect, useState } from "react";
import {
  listCrontab,
  toggleCrontabEntry,
  removeCrontabEntry,
  type CronLine,
} from "@/lib/api";
import { Button } from "@/components/ui/Button";
import { Card, CardContent } from "@/components/ui/Card";
import { Switch } from "@/components/ui/Switch";
import { Plus, RefreshCw, Trash2, AlertTriangle } from "lucide-react";

export function CrontabPage() {
  const [lines, setLines] = useState<CronLine[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      setLines(await listCrontab());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    refresh();
  }, []);

  async function onToggle(line: CronLine, enabled: boolean) {
    try {
      await toggleCrontabEntry(line.line_number, enabled);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  async function onRemove(line: CronLine) {
    if (line.kind !== "entry") return;
    if (!confirm(`Remove crontab entry?\n\n${line.raw}`)) return;
    try {
      await removeCrontabEntry(line.line_number);
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }

  const entries = lines.filter((l) => l.kind === "entry");

  return (
    <div className="flex flex-col gap-6 p-8">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">User Crontab</h1>
          <p className="text-sm text-[var(--text-muted)] mt-1">
            Entries from <code className="text-xs bg-[var(--surface-muted)] px-1.5 py-0.5 rounded">crontab -l</code>. Disabled entries are preserved as commented lines.
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="ghost" size="icon" onClick={refresh} disabled={loading} aria-label="Refresh">
            <RefreshCw className={loading ? "animate-spin h-4 w-4" : "h-4 w-4"} />
          </Button>
          <Button variant="primary" disabled>
            <Plus className="h-4 w-4" /> New entry
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

      {loading && lines.length === 0 ? (
        <p className="text-sm text-[var(--text-muted)]">Loading…</p>
      ) : entries.length === 0 ? (
        <Card>
          <CardContent className="text-sm text-[var(--text-muted)]">
            Your crontab is empty. Click <em>New entry</em> to add one.
          </CardContent>
        </Card>
      ) : (
        <Card>
          <CardContent className="p-0">
            <table className="w-full text-sm">
              <thead className="bg-[var(--surface-muted)] text-xs uppercase tracking-wider text-[var(--text-muted)]">
                <tr>
                  <th className="px-4 py-2 text-left font-medium w-12">On</th>
                  <th className="px-4 py-2 text-left font-medium">Schedule</th>
                  <th className="px-4 py-2 text-left font-medium">Command</th>
                  <th className="px-4 py-2 text-right font-medium w-20"></th>
                </tr>
              </thead>
              <tbody>
                {entries.map((line) => {
                  if (line.kind !== "entry") return null;
                  return (
                    <tr key={line.id} className="border-t border-[var(--surface-border)]">
                      <td className="px-4 py-3">
                        <Switch
                          checked={!line.disabled}
                          onCheckedChange={(v) => onToggle(line, v)}
                          aria-label="Toggle entry"
                        />
                      </td>
                      <td className="px-4 py-3 font-mono text-xs">{line.schedule}</td>
                      <td className="px-4 py-3 font-mono text-xs break-all">{line.command}</td>
                      <td className="px-4 py-3 text-right">
                        <Button variant="ghost" size="icon" onClick={() => onRemove(line)} aria-label="Remove">
                          <Trash2 className="h-3.5 w-3.5" />
                        </Button>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
