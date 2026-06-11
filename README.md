# Cronaut

A modern desktop GUI for managing **`systemd --user` timers** and the current user's
**crontab** on Debian-based Linux.

Cronaut is local-only — no telemetry, no network calls, no remote endpoints. All
state lives under `~/.config/com.alienresidents.cronaut/` and the unit files
managed by Cronaut are stored under `~/.config/systemd/user/`.

> Status: alpha. The scaffold is functional; the UI for creating/editing
> entries is wired but the create/edit forms are not yet implemented.

## Features

- **Systemd `--user` timers** — list, enable/disable, run-now, remove. Cronaut
  marks units it owns with `# Managed-By: cronaut` and refuses to modify or
  delete units lacking that marker, so your hand-written units stay safe.
- **User crontab** — list, toggle (non-destructive disable via `# CRONAUT-DISABLED:`
  prefix), remove. Comments, environment variables, and unrecognised lines are
  preserved verbatim across writes.
- **Logs** — snapshot or live-tail from `journalctl --user`, scoped to a unit
  or to the `CRON` syslog tag.
- **Themable** — light / dark / system + 7 accent colours, applied via CSS
  custom properties.
- **Tray with auto-detect** — uses `XDG_CURRENT_DESKTOP` to decide whether the
  desktop has a working StatusNotifierItem host. Falls back to taskbar on
  GNOME (which doesn't ship a tray by default).

## Tech stack

| Layer       | Choice                                                      |
| ----------- | ----------------------------------------------------------- |
| Shell       | [Tauri 2](https://tauri.app/) (Rust backend + system webview) |
| Frontend    | React 19 + TypeScript + Vite + Tailwind CSS 4               |
| Components  | Hand-rolled, shadcn-style; Radix primitives where useful    |
| Icons       | lucide-react                                                |
| Persistence | `tauri-plugin-store` (JSON in the platform config dir)      |

## Building

### Prerequisites (Debian / Ubuntu)

```bash
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev \
  libayatana-appindicator3-dev \
  build-essential curl wget file libssl-dev pkg-config

# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node + pnpm
# (use your preferred installer; tested with Node 24, pnpm 10)
```

### Dev

```bash
pnpm install
pnpm tauri dev
```

### Build a `.deb`

```bash
pnpm tauri build --bundles deb
# ./src-tauri/target/release/bundle/deb/cronaut_0.1.0_amd64.deb
sudo apt install ./src-tauri/target/release/bundle/deb/cronaut_*.deb
```

## Project layout

```
cronaut/
├── src/                          # React frontend
│   ├── components/               # UI primitives + sidebar
│   ├── features/                 # one folder per page
│   │   ├── timers/
│   │   ├── crontab/
│   │   ├── logs/
│   │   ├── settings/
│   │   └── about/
│   └── lib/                      # api wrappers, theme, utils
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── lib.rs                # Tauri Builder + plugin wiring
│       ├── timers.rs             # systemd --user CRUD
│       ├── crontab.rs            # user-crontab CRUD
│       ├── logs.rs               # journalctl streaming
│       ├── tray.rs               # tray icon + close-to-tray
│       ├── desktop_env.rs        # DE detection
│       ├── settings.rs           # typed settings schema
│       └── error.rs              # serializable error type
└── README.md
```

## Privacy

Cronaut never makes network calls. Search the source for `reqwest`, `fetch`,
`http`, `https://` — the only external URLs are the GitHub project link in the
About page (opened via `xdg-open`, only when you click it).

## Roadmap

- [ ] Create-timer and create-crontab-entry forms (currently disabled buttons)
- [ ] Edit-timer / edit-crontab-entry dialogs
- [ ] Cron expression builder UI with human-readable preview
- [ ] System cron viewer (`/etc/cron.d/*`, `/etc/crontab`) — read-only
- [ ] System systemd timers — read-only
- [ ] Autostart on login (via `tauri-plugin-autostart`)
- [ ] AppImage and Flatpak bundles
- [ ] AppArmor / seccomp profile

## License

MIT — see [LICENSE](LICENSE).
