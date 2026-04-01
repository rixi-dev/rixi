<div align="center">

<img src="docs/rixi-logo.png" alt="rixi" width="420" />

<br/>

**Because `install.sh` is not a rice manager.**

<br/>

[![version](https://img.shields.io/badge/version-0.1.0-orange?style=flat-square)](https://github.com/rixi-rs/rixi/releases)
[![license](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![built with rust](https://img.shields.io/badge/built%20with-rust-orange?style=flat-square)](https://www.rust-lang.org/)
[![linux only](https://img.shields.io/badge/linux-only-yellow?style=flat-square)]()

</div>

---

## What is RIXI?

You found a beautiful desktop on r/unixporn. You clone the repo. You spend the next hour figuring out where configs go, what fonts they used, why polybar won't start, and why your terminal looks nothing like theirs.

**RIXI fixes that.**

RIXI is a terminal-first, component-based Linux rice manager built in Rust. It lets you package, apply, switch, and roll back desktop configurations — called *rices* — in a single command. No shell scripts. No manual config copying. No guessing.

```bash
rixi apply sathiya/gruvbox
```

Your desktop transforms. Instantly.

**RIXI is v0.1** — local rice manager. Network and community features coming in v0.2+.

---

## Demo

<div align="center">
  <img src="docs/rixi-demo.gif" alt="rixi demo" width="800" />
</div>

---

## The problem with how everyone does it today

Every rice owner has a GitHub repo. Every repo has a different structure. Half of them have an `install.sh` that was written at 2am and works on exactly one machine. The other half just have a README that says "copy these files to `~/.config`."

There is no standard. There is no tooling. There is no way to try a rice without committing to it.

RIXI is the tool the ricing community never had.

---

## Commands

```bash
rixi init        # scaffold a manifest from your current setup
rixi apply       # apply a rice — snapshot, copy, reload, done
rixi rollback    # something broke? go back instantly
rixi list        # see what's installed locally
```

That's it. Four commands. Nothing else.

---

## Features

- **`rixi init`** — interactive scaffolding. Scans your system, detects installed components, asks five questions, packages your rice into a clean structured directory. Done in 30 seconds.
- **`rixi apply <author/theme>`** — snapshots your current state first, then copies configs to the right places per the built-in component registry. No paths needed in the manifest.
- **`rixi rollback`** — something broke? One command gets you back. Instantly. No drama.
- **`rixi list`** — see everything installed locally with `[current]` marked.
- **Distro-aware dependency warnings** — missing `bspwm`? rixi prints the exact `pacman`/`apt`/`dnf` command to install it.
- **29-component registry** — rixi ships knowing where every tool's config lives on your system.
- **Shell config management** — rixi can handle zsh/bash/fish prompt config as part of a rice.
- **Wallpaper handling** — feh, nitrogen, hyprpaper, swww, swaybg — rixi sets it automatically.
- **Snapshots** — every apply creates a timestamped snapshot of your previous state. Rollback is always available.

---

## Demo output

```
$ rixi init

? Theme name: gruvbox
? Author: sathiya
? Description: minimal gruvbox bspwm setup
? Color scheme: gruvbox
? Tags: minimal, dark

Scanning installed components...
  ✓ bspwm       ~/.config/bspwm/bspwmrc
  ✓ polybar     ~/.config/polybar/config
  ✓ rofi        ~/.config/rofi/config.rasi
  ✓ alacritty   ~/.config/alacritty/alacritty.toml
  ✓ picom       ~/.config/picom/picom.conf
  ✗ hyprland    not found

Scaffolded ~/.local/share/rixi/store/sathiya/gruvbox — fill in your metadata and run rixi apply sathiya/gruvbox
```

```
$ rixi apply sathiya/gruvbox

Applying sathiya/gruvbox...

Missing dependencies (install manually):
  [pacman] sudo pacman -S bspwm sxhkd polybar rofi picom feh
  [fonts]  JetBrainsMono Nerd Font — https://nerdfonts.com

Proceed anyway? [y/N] y

Snapshotting current state... done

Applying components:
  ✓ bspwm       → ~/.config/bspwm/bspwmrc
  ✓ polybar     → ~/.config/polybar/config
  ✓ rofi        → ~/.config/rofi/config.rasi
  ✓ alacritty   → ~/.config/alacritty/alacritty.toml
  ✓ picom       → ~/.config/picom/picom.conf
  ✓ wallpaper   set via feh

Applied sathiya/gruvbox. Run rixi rollback to undo.
```

```
$ rixi rollback

Rolling back to snapshot 2026-03-12T18:42:00...
  ✓ bspwm       restored
  ✓ polybar     restored
  ✓ rofi        restored
  ✓ alacritty   restored
  ✓ picom       restored

Rollback complete.
```

```
$ rixi list

Installed rices:
  sathiya/gruvbox     [current]
  sathiya/nord
  owl4ce/aesthetic
```

---

## Rice structure

Every rixi rice follows a single, predictable layout:

```
~/.local/share/rixi/store/
  sathiya/
    gruvbox/
      manifest.toml       ← the source of truth
      configs/
        bspwm/
          bspwmrc
        polybar/
          config
        rofi/
          config.rasi
        alacritty/
          alacritty.toml
        picom/
          picom.conf
      walls/
        gruvbox.png
      preview.png
```

No surprises. No guessing. rixi always knows where everything is.

---

## The manifest

```toml
[meta]
name = "gruvbox"
author = "sathiya"
version = "0.1.0"
wm = "bspwm"
display_server = ["x11"]
colorscheme = "gruvbox"
components = ["bspwm", "polybar", "rofi", "alacritty", "picom"]
tags = ["minimal", "dark", "gruvbox"]
description = "minimal gruvbox bspwm setup"

[dependencies]
packages = ["bspwm", "sxhkd", "polybar", "rofi", "alacritty", "picom", "feh"]
fonts = ["JetBrainsMono Nerd Font"]
icons = ["Papirus"]

[wallpaper]
file = "walls/gruvbox.png"
setter = "feh"
```

That's it. No file paths. No mappings. rixi ships with a built-in registry that knows where every component's config lives.

---

## Supported components

| Category | Components |
|---|---|
| WM (X11) | bspwm, i3, openbox, awesome, herbstluftwm |
| WM / Compositor (Wayland) | hyprland, sway, niri, river |
| Bars | polybar, waybar, eww |
| Launchers | rofi, wofi, tofi, fuzzel |
| Terminals | alacritty, kitty, wezterm, foot |
| Notifications | dunst, mako, swaync |
| Compositor (X11) | picom |
| Wallpaper setters | feh, nitrogen, hyprpaper, swww, swaybg |
| Lock screens | i3lock, swaylock, hyprlock |
| Shell prompts | starship |
| Keybindings | sxhkd |

Missing something? Open a PR to add it to the registry.

---

## Installation

```bash
cargo install rixi
```

Or build from source:

```bash
git clone https://github.com/rixi-rs/rixi
cd rixi
cargo build --release
sudo cp target/release/rixi /usr/local/bin/
```

**Requirements:**
- Linux (5.13+ recommended)
- Rust 1.75+

---

## Roadmap

| Version | What's coming |
|---|---|
| `v0.1` ✓ | Local rice manager — init, apply, rollback, list. Full component registry (29 components), dependency detection, shell config, wallpaper handling. |
| `v0.2` | Landlock kernel sandboxing — security-first apply |
| `v0.3` | Community themes — `rixi-dev/themes` registry. Users add themes, RIXI pulls from them. |
| `v0.4+` | TUI browser, network sync, signing |

---

## Philosophy

RIXI has opinions and is not sorry about them:

- **Terminal only.** No GUI. No Electron. No web interface. Ever.
- **Rust.** Memory safe, fast, single binary. No runtime dependencies.
- **Rollback first.** RIXI snapshots before it touches anything. You can always go back.
- **Predictable structure.** One layout. Always. Ambiguity is how things break.
- **Kernel-close.** Landlock sandboxing is coming. The kernel does the security work, not us.

---

## Contributing

rixi is early and moving fast. Issues, PRs, and feedback are all welcome.

To add a component to the registry, open a PR and add an entry to `src/registry.rs`.

To submit a theme to the v0.3+ community registry (`rixi-dev/themes`), watch this space.

---

## License

MIT — do whatever you want.

---

<div align="center">

built with 🦀 by [@sathiya](https://github.com/Sathiya-Moorthi) and the Linux ricing community

*stop copying. start RIXI.*

</div>
