//! systemd --user timer management.
//!
//! Layout on disk:
//!   ~/.config/systemd/user/<name>.timer    — schedule
//!   ~/.config/systemd/user/<name>.service  — unit invoked by the timer
//!
//! Files we manage carry the marker `# Managed-By: cronaut` in their
//! `[Unit] Description=` block context. We refuse to modify or delete
//! units that lack the marker so we never clobber hand-written units.

use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;

const MANAGED_MARKER: &str = "# Managed-By: cronaut";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    /// Unit basename without the `.timer` suffix, e.g. `backup-home`.
    pub name: String,
    pub description: String,
    /// systemd `OnCalendar=` expression, e.g. `daily`, `*-*-* 02:30:00`.
    pub schedule: String,
    /// Command line passed to `ExecStart=`.
    pub command: String,
    pub working_directory: Option<String>,
    pub enabled: bool,
    pub active: bool,
    pub last_trigger: Option<DateTime<Utc>>,
    pub next_trigger: Option<DateTime<Utc>>,
    pub managed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerSpec {
    pub name: String,
    pub description: String,
    pub schedule: String,
    pub command: String,
    pub working_directory: Option<String>,
    /// If true, also enable the timer right after writing it.
    #[serde(default = "default_true")]
    pub enable: bool,
}

fn default_true() -> bool {
    true
}

fn user_systemd_dir() -> Result<PathBuf> {
    let base =
        dirs::config_dir().ok_or_else(|| AppError::not_found("config_dir not resolvable"))?;
    Ok(base.join("systemd").join("user"))
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(AppError::invalid("name must not be empty"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(AppError::invalid("name may only contain [A-Za-z0-9._-]"));
    }
    if name.starts_with('-') || name.starts_with('.') {
        return Err(AppError::invalid("name must not start with - or ."));
    }
    Ok(())
}

async fn run(cmd: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(cmd).args(args).output().await?;
    if !out.status.success() {
        return Err(AppError::Command {
            cmd: format!("{cmd} {}", args.join(" ")),
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        });
    }
    Ok(String::from_utf8(out.stdout)?)
}

async fn systemctl(args: &[&str]) -> Result<String> {
    let mut all = vec!["--user"];
    all.extend_from_slice(args);
    run("systemctl", &all).await
}

#[tauri::command]
pub async fn list_timers() -> Result<Vec<Timer>> {
    // 1. List active+inactive --user timers in JSON form.
    let raw = systemctl(&["list-timers", "--all", "--no-pager", "--output=json"])
        .await
        .unwrap_or_default();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&raw).unwrap_or_default();

    let dir = user_systemd_dir()?;

    let mut timers = Vec::new();
    for entry in entries {
        let unit = entry.get("unit").and_then(|v| v.as_str()).unwrap_or("");
        if !unit.ends_with(".timer") {
            continue;
        }
        let name = unit.trim_end_matches(".timer").to_string();
        let timer_path = dir.join(format!("{name}.timer"));
        let service_path = dir.join(format!("{name}.service"));

        let (description, schedule, working_directory, command, managed) =
            read_unit_details(&timer_path, &service_path).unwrap_or_default();

        let next_trigger = entry
            .get("next")
            .and_then(|v| v.as_str())
            .and_then(parse_systemctl_time);
        let last_trigger = entry
            .get("last")
            .and_then(|v| v.as_str())
            .and_then(parse_systemctl_time);

        let enabled = is_enabled(&name).await.unwrap_or(false);
        let active = is_active(&name).await.unwrap_or(false);

        timers.push(Timer {
            name,
            description,
            schedule,
            command,
            working_directory,
            enabled,
            active,
            last_trigger,
            next_trigger,
            managed,
        });
    }
    Ok(timers)
}

async fn is_enabled(name: &str) -> Result<bool> {
    let out = Command::new("systemctl")
        .args(["--user", "is-enabled", &format!("{name}.timer")])
        .output()
        .await?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .trim()
        .eq_ignore_ascii_case("enabled"))
}

async fn is_active(name: &str) -> Result<bool> {
    let out = Command::new("systemctl")
        .args(["--user", "is-active", &format!("{name}.timer")])
        .output()
        .await?;
    Ok(String::from_utf8_lossy(&out.stdout).trim() == "active")
}

fn read_unit_details(
    timer: &Path,
    service: &Path,
) -> Result<(String, String, Option<String>, String, bool)> {
    let timer_src = std::fs::read_to_string(timer).unwrap_or_default();
    let service_src = std::fs::read_to_string(service).unwrap_or_default();
    let managed = timer_src.contains(MANAGED_MARKER) || service_src.contains(MANAGED_MARKER);
    let description = grep_value(&service_src, "Description").unwrap_or_default();
    let schedule = grep_value(&timer_src, "OnCalendar").unwrap_or_default();
    let command = grep_value(&service_src, "ExecStart").unwrap_or_default();
    let wd = grep_value(&service_src, "WorkingDirectory");
    Ok((description, schedule, wd, command, managed))
}

fn grep_value(src: &str, key: &str) -> Option<String> {
    src.lines()
        .filter_map(|l| {
            let trimmed = l.trim_start();
            if trimmed.starts_with('#') || trimmed.starts_with(';') {
                return None;
            }
            let (k, v) = trimmed.split_once('=')?;
            if k.trim().eq_ignore_ascii_case(key) {
                Some(v.trim().to_string())
            } else {
                None
            }
        })
        .next()
}

fn parse_systemctl_time(s: &str) -> Option<DateTime<Utc>> {
    if s == "n/a" || s.is_empty() {
        return None;
    }
    // systemctl JSON returns microseconds since epoch as a number, but the
    // `--output=json` form yields strings like "Wed 2026-06-11 09:00:00 AEST"
    // and integer micros depending on systemd version. Try integer first.
    if let Ok(micros) = s.parse::<i64>() {
        return DateTime::<Utc>::from_timestamp_micros(micros);
    }
    None
}

#[tauri::command]
pub async fn create_timer(spec: TimerSpec) -> Result<()> {
    validate_name(&spec.name)?;
    if spec.schedule.trim().is_empty() {
        return Err(AppError::invalid("schedule must not be empty"));
    }
    if spec.command.trim().is_empty() {
        return Err(AppError::invalid("command must not be empty"));
    }
    let dir = user_systemd_dir()?;
    tokio::fs::create_dir_all(&dir).await?;
    let timer_path = dir.join(format!("{}.timer", spec.name));
    let service_path = dir.join(format!("{}.service", spec.name));
    if timer_path.exists() || service_path.exists() {
        return Err(AppError::invalid(format!(
            "unit {} already exists; use update_timer to modify",
            spec.name
        )));
    }
    write_units(&spec, &timer_path, &service_path).await?;
    systemctl(&["daemon-reload"]).await?;
    if spec.enable {
        systemctl(&["enable", "--now", &format!("{}.timer", spec.name)]).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn update_timer(spec: TimerSpec) -> Result<()> {
    validate_name(&spec.name)?;
    let dir = user_systemd_dir()?;
    let timer_path = dir.join(format!("{}.timer", spec.name));
    let service_path = dir.join(format!("{}.service", spec.name));
    require_managed(&timer_path)?;
    require_managed(&service_path)?;
    write_units(&spec, &timer_path, &service_path).await?;
    systemctl(&["daemon-reload"]).await?;
    if spec.enable {
        systemctl(&["restart", &format!("{}.timer", spec.name)]).await?;
    }
    Ok(())
}

fn require_managed(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(AppError::not_found(format!(
            "{} does not exist",
            path.display()
        )));
    }
    let body = std::fs::read_to_string(path)?;
    if !body.contains(MANAGED_MARKER) {
        return Err(AppError::invalid(format!(
            "refusing to touch {}: not managed by cronaut (add `{MANAGED_MARKER}` to take ownership)",
            path.display()
        )));
    }
    Ok(())
}

async fn write_units(spec: &TimerSpec, timer_path: &Path, service_path: &Path) -> Result<()> {
    let service = format!(
        "{MANAGED_MARKER}\n\
        [Unit]\n\
        Description={desc}\n\
        \n\
        [Service]\n\
        Type=oneshot\n\
        ExecStart={cmd}\n{wd}\
        \n\
        [Install]\n\
        WantedBy=default.target\n",
        desc = spec.description,
        cmd = spec.command,
        wd = spec
            .working_directory
            .as_ref()
            .map(|w| format!("WorkingDirectory={w}\n"))
            .unwrap_or_default(),
    );
    let timer = format!(
        "{MANAGED_MARKER}\n\
        [Unit]\n\
        Description=Timer for {name}\n\
        \n\
        [Timer]\n\
        OnCalendar={cal}\n\
        Persistent=true\n\
        Unit={name}.service\n\
        \n\
        [Install]\n\
        WantedBy=timers.target\n",
        name = spec.name,
        cal = spec.schedule,
    );
    tokio::fs::write(service_path, service).await?;
    tokio::fs::write(timer_path, timer).await?;
    Ok(())
}

#[tauri::command]
pub async fn enable_timer(name: String) -> Result<()> {
    validate_name(&name)?;
    systemctl(&["enable", "--now", &format!("{name}.timer")]).await?;
    Ok(())
}

#[tauri::command]
pub async fn disable_timer(name: String) -> Result<()> {
    validate_name(&name)?;
    systemctl(&["disable", "--now", &format!("{name}.timer")]).await?;
    Ok(())
}

#[tauri::command]
pub async fn run_timer_now(name: String) -> Result<()> {
    validate_name(&name)?;
    systemctl(&["start", &format!("{name}.service")]).await?;
    Ok(())
}

#[tauri::command]
pub async fn remove_timer(name: String) -> Result<()> {
    validate_name(&name)?;
    let dir = user_systemd_dir()?;
    let timer_path = dir.join(format!("{name}.timer"));
    let service_path = dir.join(format!("{name}.service"));
    require_managed(&timer_path)?;
    require_managed(&service_path)?;
    let _ = systemctl(&["disable", "--now", &format!("{name}.timer")]).await;
    tokio::fs::remove_file(&timer_path).await?;
    tokio::fs::remove_file(&service_path).await?;
    systemctl(&["daemon-reload"]).await?;
    Ok(())
}
