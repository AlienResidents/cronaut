//! User crontab CRUD via `crontab -l` / `crontab -`.
//!
//! Disabled entries are stored as `# CRONAUT-DISABLED: <original line>` so we
//! never delete the user's data implicitly. Comments, env-var lines, and
//! malformed lines are preserved verbatim across writes.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

const DISABLE_PREFIX: &str = "# CRONAUT-DISABLED: ";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CronLine {
    Entry {
        /// Stable id derived from line index. Re-issued on every list call.
        id: String,
        line_number: usize,
        schedule: String,
        command: String,
        disabled: bool,
        raw: String,
    },
    Passthrough {
        id: String,
        line_number: usize,
        raw: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronEntrySpec {
    pub schedule: String,
    pub command: String,
}

#[tauri::command]
pub async fn list_crontab() -> Result<Vec<CronLine>> {
    let raw = read_crontab().await?;
    Ok(parse(&raw))
}

#[tauri::command]
pub async fn add_crontab_entry(spec: CronEntrySpec) -> Result<()> {
    validate_spec(&spec)?;
    let mut lines = parse(&read_crontab().await?);
    lines.push(CronLine::Entry {
        id: format!("e{}", lines.len() + 1),
        line_number: lines.len() + 1,
        schedule: spec.schedule.clone(),
        command: spec.command.clone(),
        disabled: false,
        raw: format!("{} {}", spec.schedule, spec.command),
    });
    write_crontab(&render(&lines)).await
}

#[tauri::command]
pub async fn update_crontab_entry(line_number: usize, spec: CronEntrySpec) -> Result<()> {
    validate_spec(&spec)?;
    let mut lines = parse(&read_crontab().await?);
    let idx = locate(&lines, line_number)?;
    if let CronLine::Entry {
        disabled,
        raw,
        schedule,
        command,
        ..
    } = &mut lines[idx]
    {
        *schedule = spec.schedule.clone();
        *command = spec.command.clone();
        *raw = if *disabled {
            format!("{}{} {}", DISABLE_PREFIX, spec.schedule, spec.command)
        } else {
            format!("{} {}", spec.schedule, spec.command)
        };
    } else {
        return Err(AppError::invalid("line is not a managed entry"));
    }
    write_crontab(&render(&lines)).await
}

#[tauri::command]
pub async fn toggle_crontab_entry(line_number: usize, enabled: bool) -> Result<()> {
    let mut lines = parse(&read_crontab().await?);
    let idx = locate(&lines, line_number)?;
    if let CronLine::Entry {
        disabled,
        raw,
        schedule,
        command,
        ..
    } = &mut lines[idx]
    {
        *disabled = !enabled;
        *raw = if *disabled {
            format!("{}{} {}", DISABLE_PREFIX, schedule, command)
        } else {
            format!("{} {}", schedule, command)
        };
    } else {
        return Err(AppError::invalid("line is not a managed entry"));
    }
    write_crontab(&render(&lines)).await
}

#[tauri::command]
pub async fn remove_crontab_entry(line_number: usize) -> Result<()> {
    let mut lines = parse(&read_crontab().await?);
    let idx = locate(&lines, line_number)?;
    if !matches!(lines[idx], CronLine::Entry { .. }) {
        return Err(AppError::invalid("line is not a managed entry"));
    }
    lines.remove(idx);
    write_crontab(&render(&lines)).await
}

#[tauri::command]
pub fn validate_cron_expression(expr: String) -> Result<()> {
    // The standard 5-field user-crontab form. The `cron` crate uses the
    // 6-field Quartz form, so we prepend a "0" seconds field for parsing.
    let normalised = format!("0 {}", expr.trim());
    use std::str::FromStr;
    cron::Schedule::from_str(&normalised)
        .map(|_| ())
        .map_err(|e| AppError::parse(format!("invalid cron expression: {e}")))
}

// --- internals -----------------------------------------------------

fn validate_spec(spec: &CronEntrySpec) -> Result<()> {
    if spec.schedule.trim().is_empty() {
        return Err(AppError::invalid("schedule must not be empty"));
    }
    if spec.command.trim().is_empty() {
        return Err(AppError::invalid("command must not be empty"));
    }
    use std::str::FromStr;
    let normalised = format!("0 {}", spec.schedule.trim());
    cron::Schedule::from_str(&normalised)
        .map_err(|e| AppError::parse(format!("invalid cron expression: {e}")))?;
    if spec.command.contains('\n') {
        return Err(AppError::invalid("command must be a single line"));
    }
    Ok(())
}

fn locate(lines: &[CronLine], line_number: usize) -> Result<usize> {
    lines
        .iter()
        .position(|l| match l {
            CronLine::Entry {
                line_number: ln, ..
            } => *ln == line_number,
            CronLine::Passthrough {
                line_number: ln, ..
            } => *ln == line_number,
        })
        .ok_or_else(|| AppError::not_found(format!("line {line_number} not found")))
}

fn parse(src: &str) -> Vec<CronLine> {
    let mut out = Vec::new();
    for (i, raw) in src.lines().enumerate() {
        let ln = i + 1;
        let id = format!("l{ln}");
        // Disabled entry?
        if let Some(rest) = raw.strip_prefix(DISABLE_PREFIX) {
            if let Some((sched, cmd)) = split_entry(rest) {
                out.push(CronLine::Entry {
                    id,
                    line_number: ln,
                    schedule: sched,
                    command: cmd,
                    disabled: true,
                    raw: raw.to_string(),
                });
                continue;
            }
        }
        // Comment / env / blank?
        let trimmed = raw.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || is_env_assignment(trimmed) {
            out.push(CronLine::Passthrough {
                id,
                line_number: ln,
                raw: raw.to_string(),
            });
            continue;
        }
        // Live entry?
        if let Some((sched, cmd)) = split_entry(raw) {
            out.push(CronLine::Entry {
                id,
                line_number: ln,
                schedule: sched,
                command: cmd,
                disabled: false,
                raw: raw.to_string(),
            });
        } else {
            out.push(CronLine::Passthrough {
                id,
                line_number: ln,
                raw: raw.to_string(),
            });
        }
    }
    out
}

fn split_entry(raw: &str) -> Option<(String, String)> {
    let trimmed = raw.trim_start();
    // Special "@reboot", "@daily" etc.
    if let Some(rest) = trimmed.strip_prefix('@') {
        let mut parts = rest.splitn(2, char::is_whitespace);
        let kw = parts.next()?;
        let cmd = parts.next()?.trim();
        if cmd.is_empty() {
            return None;
        }
        return Some((format!("@{kw}"), cmd.to_string()));
    }
    // Standard 5-field
    let mut tokens = trimmed.split_whitespace();
    let mut fields = Vec::with_capacity(5);
    for _ in 0..5 {
        fields.push(tokens.next()?.to_string());
    }
    let cmd = tokens.collect::<Vec<&str>>().join(" ");
    if cmd.is_empty() {
        return None;
    }
    Some((fields.join(" "), cmd))
}

fn is_env_assignment(line: &str) -> bool {
    // crontab(5): VAR = value  /  VAR=value
    if let Some((lhs, _)) = line.split_once('=') {
        let lhs = lhs.trim();
        !lhs.is_empty()
            && lhs.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && lhs
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic() || c == '_')
                .unwrap_or(false)
    } else {
        false
    }
}

fn render(lines: &[CronLine]) -> String {
    let mut out = String::new();
    for l in lines {
        let raw = match l {
            CronLine::Entry { raw, .. } => raw,
            CronLine::Passthrough { raw, .. } => raw,
        };
        out.push_str(raw);
        out.push('\n');
    }
    out
}

async fn read_crontab() -> Result<String> {
    let out = Command::new("crontab").arg("-l").output().await?;
    if !out.status.success() {
        // Empty crontab returns exit 1 with "no crontab for <user>" on stderr.
        let stderr = String::from_utf8_lossy(&out.stderr);
        if stderr.contains("no crontab for") {
            return Ok(String::new());
        }
        return Err(AppError::Command {
            cmd: "crontab -l".into(),
            code: out.status.code(),
            stderr: stderr.into_owned(),
        });
    }
    Ok(String::from_utf8(out.stdout)?)
}

async fn write_crontab(body: &str) -> Result<()> {
    let mut child = Command::new("crontab")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(body.as_bytes()).await?;
        stdin.flush().await?;
        drop(stdin);
    }
    let out = child.wait_with_output().await?;
    if !out.status.success() {
        return Err(AppError::Command {
            cmd: "crontab -".into(),
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }
    let _ = std::io::stdout().flush();
    Ok(())
}
