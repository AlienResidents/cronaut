# Security

Cronaut is a **local-only desktop application**. It does not make network
calls, does not send telemetry, does not expose any listening port, and
does not accept untrusted IPC. All persistent state lives in the user's
own `~/.config/com.alienresidents.cronaut/` directory; managed unit files
live under `~/.config/systemd/user/`.

The threat model is correspondingly narrow: the primary concerns are
local privilege/integrity (don't corrupt the user's crontab, don't touch
units we didn't author) and crash-resistance.

## Reporting a vulnerability

If you find a security issue in Cronaut itself, please:

1. **Preferred:** open a private vulnerability advisory at
   <https://github.com/AlienResidents/cronaut/security/advisories/new>.
2. **Alternative:** email `chris@akuru.com.au` with the subject prefix
   `[cronaut security]`.

Please don't open public issues for unpatched security bugs.

We aim to acknowledge reports within five working days.

## Supported versions

Cronaut is in alpha. Only the latest released `0.x.y` version is
supported with security fixes; older `0.x.y` releases will not be
back-patched. Once Cronaut reaches `1.0.0`, this section will be
revisited.

| Version  | Supported          |
| -------- | ------------------ |
| `0.1.x`  | :white_check_mark: |
| `< 0.1`  | :x:                |

## Dismissed advisories

The advisories below have been triaged and **intentionally not patched
in Cronaut**. Each entry records the dependency chain, why we can't fix
it ourselves, and the conditions under which it should be re-opened.

### GHSA-wrw7-89jp-8q8g

**Package:** `glib` (Rust crate, gtk-rs)
**Affected range:** `>= 0.15.0, < 0.20.0`
**Pinned in cronaut at:** `0.18.5` (transitive)
**Severity:** medium
**Dismissed:** 2026-06-12 with reason `tolerable_risk`

**Summary.** Unsoundness in the `Iterator` and `DoubleEndedIterator`
implementations for `glib::VariantStrIter`. `VariantStrIter::impl_get`
passes `&p` (immutable) where `&mut p` is required to a variadic C
function (`g_variant_get_child`) that mutates the pointer in place.
Newer Rust optimisers elide the unsound write entirely, leaving the
pointer at NULL and causing a NULL-pointer dereference on iteration.
Fixed in `glib 0.20.0` via [gtk-rs/gtk-rs-core#1343](https://github.com/gtk-rs/gtk-rs-core/pull/1343).

**Dependency chain that pulls `glib 0.18.5` in:**

```
glib 0.18.5
  ← atk 0.18.2
  ← gtk 0.18.2
  ← libappindicator 0.9.0
  ← tray-icon 0.23.1
  ← tauri 2.11.2 (+ wry 0.55.1, tao 0.35.3, webkit2gtk 2.0.2)
```

**Why this can't be fixed inside Cronaut:**

- gtk-rs 0.18 is the last of the gtk3 binding line. The gtk-rs project
  has stopped releasing gtk3 bindings (max stable: `gtk 0.18.2`,
  `atk 0.18.2`, both unchanged since late 2024). The `glib 0.18` line
  ended at `0.18.5` with no backport of the soundness fix — the patch
  landed only in `glib 0.20.0`.
- `wry 0.55.1`, `tao 0.35.3`, and `tray-icon 0.23.1` all hard-pin
  `gtk ^0.18` for Linux/*BSD targets and offer no `gtk4` / `webkit6`
  feature flag. Cronaut is already on the latest published version of
  each.
- Tauri's maintainers triaged the same advisory class
  (`GHSA-wrw7-89jp-8q8g`) as `not_planned` for Tauri 1.x and 2.x:
  [tauri-apps/tauri#12048](https://github.com/tauri-apps/tauri/issues/12048).
- The gtk4 migration is tracked but not merged or released — it lands
  in Tauri 3.x:
  - [tauri-apps/tauri#7335](https://github.com/tauri-apps/tauri/issues/7335)
    — `[feat] Migrate to GTK4`
  - [tauri-apps/tauri#14684](https://github.com/tauri-apps/tauri/issues/14684)
    — `feat(linux): migrate to GTK4 and WebKitGTK 6.0`
  - [tauri-apps/tauri#12561](https://github.com/tauri-apps/tauri/issues/12561),
    [#12562](https://github.com/tauri-apps/tauri/issues/12562),
    [#12563](https://github.com/tauri-apps/tauri/issues/12563)
    — split work for `tauri-runtime-wry` / `tauri-runtime` / `tauri`
    onto gtk4-rs
  - [tauri-apps/wry#1474](https://github.com/tauri-apps/wry/issues/1474)
    — `Upgrade wry to gtk4-rs and webkit6`
  - [tauri-apps/wry#1530](https://github.com/tauri-apps/wry/issues/1530)
    — `fix(linux): Port to webkitgtk6`

Patching `[patch.crates-io] glib = ...` to a fork that backports the
fix is technically possible but requires us to maintain that fork, and
adds an integration risk worse than the unsoundness it would remove
(no public fork exists; we'd be running an internal divergent glib
under unreleased gtk-rs callsites).

**Risk profile for Cronaut specifically:**

- The unsound function is `glib::VariantStrIter::impl_get`, reachable
  only when iterating GLib `VARIANT` string-array values via D-Bus.
  Cronaut's own code never touches glib directly. Any reachability
  is via Tauri tray / appindicator internals.
- Failure mode is a NULL-pointer crash, not RCE or privilege
  escalation. Severity is "medium" for that reason.
- Cronaut is local-only (no network surface, no remote inputs, no
  untrusted IPC). Any crash from this path is user-visible and
  recoverable, not a security boundary breach.

**Mitigations and re-evaluation triggers:**

- Stay current on Tauri 2.x point releases so any cherry-picked
  workarounds land here automatically.
- Re-open and re-evaluate when Tauri 3.x releases with the gtk4 +
  webkit6 + non-libappindicator-gtk3 stack. At that point the fix is
  a single `tauri` version bump and `cargo update -p glib`.
- This dismissal applies **only** to `GHSA-wrw7-89jp-8q8g` reached via
  the gtk3 chain above. Any future glib advisory, or a glib advisory
  reachable via a different path, must be re-triaged from scratch.
