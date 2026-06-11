//! Read and tail logs.
//!
//! Sources:
//!   - systemd --user units    → `journalctl --user -u <unit>`
//!   - user cron               → `journalctl --user-unit=cron* -t CRON`
//!                               (best-effort; what's available depends on
//!                                whether cron is configured to log via
//!                                syslog/journald on this distro)
//!
//! Two modes:
//!   - one-shot fetch (snapshot)      → `fetch_logs`
//!   - background tail (events)       → `start_tail` / `stop_tail`
//!     Emits `log:line:<id>` and ends with `log:end:<id>`.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LogSource {
    /// A specific systemd --user unit, e.g. "backup-home.service".
    Unit { unit: String },
    /// Cron output across all user-cron-related units.
    UserCron,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchSpec {
    pub source: LogSource,
    /// Number of journal lines to fetch (most recent N).
    pub lines: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailSpec {
    pub id: String,
    pub source: LogSource,
    pub lines: u32,
}

pub struct LogState {
    children: Mutex<Vec<(String, Child)>>,
}

impl LogState {
    pub fn new() -> Self {
        Self {
            children: Mutex::new(Vec::new()),
        }
    }
}

fn build_args(source: &LogSource, lines: u32, follow: bool) -> Vec<String> {
    let mut args = vec![
        "--user".to_string(),
        "--no-pager".to_string(),
        "--output=short-iso".to_string(),
        format!("--lines={lines}"),
    ];
    if follow {
        args.push("--follow".to_string());
    }
    match source {
        LogSource::Unit { unit } => {
            args.push("-u".into());
            args.push(unit.clone());
        }
        LogSource::UserCron => {
            args.push("-t".into());
            args.push("CRON".into());
        }
    }
    args
}

#[tauri::command]
pub async fn fetch_logs(spec: FetchSpec) -> Result<Vec<String>> {
    let args = build_args(&spec.source, spec.lines, false);
    let out = Command::new("journalctl").args(&args).output().await?;
    if !out.status.success() {
        return Err(AppError::Command {
            cmd: format!("journalctl {}", args.join(" ")),
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.to_string())
        .collect())
}

#[tauri::command]
pub async fn start_tail(
    app: AppHandle,
    state: tauri::State<'_, LogState>,
    spec: TailSpec,
) -> Result<()> {
    let args = build_args(&spec.source, spec.lines, true);
    let mut child = Command::new("journalctl")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::invalid("journalctl produced no stdout"))?;

    let id_for_task = spec.id.clone();
    let app_for_task = app.clone();
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = app_for_task.emit(&format!("log:line:{}", id_for_task), line);
        }
        let _ = app_for_task.emit(&format!("log:end:{}", id_for_task), ());
    });

    state
        .children
        .lock()
        .expect("log state poisoned")
        .push((spec.id, child));
    Ok(())
}

#[tauri::command]
pub async fn stop_tail(state: tauri::State<'_, LogState>, id: String) -> Result<()> {
    let child = {
        let mut guard = state.children.lock().expect("log state poisoned");
        guard
            .iter()
            .position(|(cid, _)| cid == &id)
            .map(|idx| guard.remove(idx).1)
    };
    if let Some(mut c) = child {
        let _ = c.start_kill();
        let _ = c.wait().await;
    }
    Ok(())
}
