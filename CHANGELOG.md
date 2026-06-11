# Changelog

All notable changes to this project will be documented here. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-12

First alpha. Functional scaffold; create/edit forms for timers and crontab
entries are not yet implemented.

### Added
- Initial scaffold (Tauri 2 + React 19 + Tailwind 4)
- Systemd `--user` timer listing, enable/disable, run-now, remove
- User crontab listing, toggle (non-destructive), remove
- `journalctl --user` log snapshots and live tail
- Theme system (light/dark/system + 7 accents)
- System tray with auto-detect; taskbar fallback when no native tray host
- Single-instance launch
- MIT license, README, CI (typecheck, fmt, clippy, test), release workflow (.deb)

### Known limitations
- Create / edit forms for timers and crontab entries are not yet implemented
  (the buttons are present but disabled).
- System-level cron and timers are not exposed.
