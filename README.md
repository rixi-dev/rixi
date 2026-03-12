<p align="center">

<img src="docs/rixi-logo.png" alt="rixi" width="420"/>

<br>

**Stop copying. Start RIXI.**

<br/>

[![version](https://img.shields.io/badge/version-0.1.0-orange?style=flat-square)](https://github.com/rixi-dev/rixi/releases) [![license](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE) [![built with rust](https://img.shields.io/badge/built%20with-rust-orange?style=flat-square)](https://www.rust-lang.org/) [![linux only](https://img.shields.io/badge/linux-only-yellow?style=flat-square)]()

</p>

------------------------------------------------------------------------

## What is RIXI?

You found a beautiful desktop on r/unixporn. You clone the repo. You spend the next hour figuring out where configs go, what fonts they used, why polybar won't start, and why your terminal looks nothing like theirs.

**RIXI fixes that.**

RIXI is a terminal-first, component-based Linux rice manager built in Rust. It lets you package, apply, switch, and roll back desktop configurations (called *rices*) in a single command. No shell scripts. No manual config copying. No guessing.

``` bash
rixi apply sathiya/gruvbox
```

Your desktop transforms. Instantly.

> **RIXI is v0.1** - a local rice manager. Network and community features are coming in v0.2+.

------------------------------------------------------------------------

## The problem with how everyone does it today

Every rice owner has a GitHub repo. Every repo has a different structure. Half of them have an `install.sh` that was written at 2am and works on exactly one machine. The other half just have a README that says "copy these files to `~/.config`."

There is no standard. There is no tooling. There is no easy way to try a rice without committing to it.

RIXI aims to be the tool the ricing community never had.

------------------------------------------------------------------------

<p align="center">

<img src="docs/rixi-demo.gif" alt="rixi demo" width="800"/>

</p>

------------------------------------------------------------------------

## Commands

``` bash
rixi init        # scaffold a manifest from your current setup
rixi apply       # snapshot, copy configs, reload, done
rixi rollback    # something broke? go back instantly
rixi list        # see what's installed locally
```

Four commands. That's it.

### What each one does

**`rixi init`** scans your system, detects installed components, asks five questions, and packages your rice into a clean structured directory. Done in 30 seconds.

**`rixi apply <author/theme>`** snapshots your current state first, then copies configs to the right places using the built-in component registry. No paths needed in the manifest.

**`rixi rollback`** reverts to the last snapshot instantly. One command. No drama.

**`rixi list`** shows everything installed locally with `[current]` marked.

------------------------------------------------------------------------

## Features

-   **Distro-aware dependency warnings.** Missing `bspwm`? RIXI prints the exact `pacman`/`apt`/`dnf` command to install it.
-   **29-component registry.** RIXI ships knowing where every supported tool's config lives on your system.
-   **Shell config management.** RIXI handles zsh/bash/fish prompt config as part of a rice.
-   **Wallpaper handling.** feh, nitrogen, hyprpaper, swww, swaybg — RIXI sets it automatically.
-   **Timestamped snapshots.** Every apply creates a snapshot of your previous state. Rollback is always one command away.

------------------------------------------------------------------------

## Demo output

``` bash         
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

``` bash        
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

``` bash       
$ rixi rollback

Rolling back to snapshot 2026-03-12T18:42:00...
  ✓ bspwm       restored
  ✓ polybar     restored
  ✓ rofi        restored
  ✓ alacritty   restored
  ✓ picom       restored

Rollback complete.
```

``` bash     
$ rixi list

Installed rices:
  sathiya/gruvbox     [current]
  sathiya/sands
  owl4ce/dusk
```

------------------------------------------------------------------------

## Rice structure

Every RIXI rice follows a single, predictable layout:

``` bash         
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

No surprises. No guessing. RIXI always knows where everything is.

------------------------------------------------------------------------

## The manifest

``` toml
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

No file paths. No custom mappings. RIXI ships with a built-in registry that knows where every component's config lives.

------------------------------------------------------------------------

## Supported components

| Category                  | Components                                |
|---------------------------|-------------------------------------------|
| WM (X11)                  | bspwm, i3, openbox, awesome, herbstluftwm |
| WM / Compositor (Wayland) | hyprland, sway, niri, river               |
| Bars                      | polybar, waybar, eww                      |
| Launchers                 | rofi, wofi, tofi, fuzzel                  |
| Terminals                 | alacritty, kitty, wezterm, foot           |
| Notifications             | dunst, mako, swaync                       |
| Compositor (X11)          | picom                                     |
| Wallpaper setters         | feh, nitrogen, hyprpaper, swww, swaybg    |
| Lock screens              | i3lock, swaylock, hyprlock                |
| Shell prompts             | starship                                  |
| Keybindings               | sxhkd                                     |

Missing something? [Open a PR](https://github.com/rixi-dev/rixi) to add it to the registry.

------------------------------------------------------------------------

## Installation

Build from source:

``` bash
git clone https://github.com/rixi-dev/rixi
cd rixi
cargo build --release
chmod +x target/release/rixi
sudo cp target/release/rixi /usr/local/bin/
```

------------------------------------------------------------------------

## Roadmap

-   [x] **v0.1** - Local rice manager: init, apply, rollback, list. Full 29-component registry, dependency detection, shell config, wallpaper handling.

-   [ ] **v0.2** - Community registry via `rixi-dev/themes`. `rixi pull <author/theme>` downloads a rice into your local store. `rixi push <author/theme>` opens a GitHub PR against the community registry directly in your browser.

-   [ ] **v0.3** - TUI browser, search, ratings.

## Contributing

RIXI is early and moving fast. Issues, PRs, and feedback are all .welcome.

To submit a theme to the v0.3+ community registry (`rixi-dev/themes`), watch this space.

------------------------------------------------------------------------

## License

[MIT](LICENSE).

------------------------------------------------------------------------
