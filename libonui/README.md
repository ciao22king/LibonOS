# LibonUI

The desktop shell of **LibonOS**: the first-run setup, the login screen, the
session, the Store, and `libon` — the package and update manager.

Three Rust crates:

| crate | what it is |
| --- | --- |
| `libonui-core` | Everything with a rule in it: design tokens, versions and the update policy, accounts, the window manager, the package backends. No UI toolkit. |
| `libonui-app` | The shell itself — `egui`/`eframe`. Setup, login, desktop, Store, Settings, terminal, file manager, text editor. |
| `libonui-cli` | `libon`, the command-line twin of the Store and the Updates pane. |

```text
cargo run -p libonui-app            # setup on a fresh machine, otherwise login
cargo run -p libonui-app -- --demo  # straight into a populated desktop
cargo run -p libonui-cli -- status  # libon
cargo test --workspace
```

## First boot

Setup asks for four things, one screen at a time: your **first name and
password**, the **package manager** to use, and whether to send telemetry. The
name and password become the local account; the password is stored as a salted
PBKDF2-HMAC-SHA256 digest and never in clear. The login screen afterwards checks
against that digest.

The account, the settings and the session live under the user's state directory,
so `--state <dir>` gives you a throwaway machine to try things in.

## The package manager choice

Three backends, one interface (`PackageManager` in `libonui-core`):

* **libon** — preinstalled and the default. `libon install <app>` resolves the
  name to a publisher's AppImage, downloads it to `~/Applications`, and writes a
  `.desktop` entry. `libon update` updates in place.
* **APT** — Debian and Ubuntu repositories.
* **pacman** — Arch.

The choice is recorded in Settings and honoured by both the Store and the CLI. A
backend that is not installed on the machine refuses the install and says so
rather than appearing to work: libon is always present, APT and pacman may not
be.

## Updates, and why there are two kinds

LibonOS separates two operations that other systems blur together:

* **In-place update** (`1.0.0` → `1.0.1`, `1.1.0`) — patches and minor releases.
  Available from Settings *and* from the terminal with `libon update`.
* **Full system upgrade** (`1.0.0` → `2.0.0`) — rewrites the system image. It is
  offered **only from Settings › Aggiornamenti**, behind a confirmation that says
  what it will do. `libon upgrade` explains the rule instead of running it.

That asymmetry is the point: a stray command should never be able to rebuild the
machine underneath the user. The policy itself lives in `libonui-core::version`,
so the UI and the CLI cannot drift apart.

## What a good OS ships with

`libonui-core::system` is the single source of truth. It lists 16 preinstalled
apps — Firefox first and pinned to the dock, then Files, Terminal, Store,
Settings, and the rest — plus a 14-point capability checklist (browser, file
management, terminal, Store, updates, network, audio, Bluetooth, printing, power,
accessibility, security, backup, security updates) that Settings reports on.

## Screens

`libonui --shots <dir>` renders every screen to PNG without a display server and
writes an `index.html` contact sheet. The images in `artifacts/` are produced
that way, so the screenshots in the docs are the real rendered output rather than
a mock-up:

```text
xvfb-run -a -s "-screen 0 1440x900x24" cargo run -p libonui-app -- --shots artifacts
```

## Design

A deep violet-black canvas with one violet-to-cyan accent pair. Colour carries
meaning — action, focus, status — never decoration. The tokens are data
(`libonui-core::tokens`), so the shell, the installer, the terminal and the CLI
all read the same values, and the contrast ratios are testable.
