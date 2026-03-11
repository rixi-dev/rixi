# rixi v0.2 — Component Support, Dependency Handling & Shell Configuration

## Overview

v0.2 builds on top of the working v0.1 local rice manager. It adds:
- Full component support for both X11 and Wayland ecosystems
- Distro-aware dependency detection and reporting
- Safe shell configuration management (zsh, bash, fish)
- Font detection via fc-list
- A community-maintained external component index (components.toml)

No network, no TUI, no Landlock yet. Those are v0.3+.

---

## External Component Index

rixi ships with a built-in component registry embedded in the binary, but also
fetches and caches an external index from the rixi-rs GitHub org:

  ~/.local/share/rixi/components.toml

This file is fetched on first run and cached locally. It is the source of truth
for component config paths, reload commands, and display server compatibility.
If the file is absent or stale (older than 7 days), rixi fetches a fresh copy.
If fetch fails, rixi falls back to the built-in registry.

The external index structure:

```toml
[bspwm]
config_paths = ["~/.config/bspwm/bspwmrc"]
reload = "bspc wm -r"
display_server = "x11"
packages = ["bspwm", "sxhkd"]

[polybar]
config_paths = ["~/.config/polybar/config", "~/.config/polybar/colors.ini"]
reload = "killall -q polybar; polybar &"
display_server = "x11"
packages = ["polybar"]

[hyprland]
config_paths = ["~/.config/hypr/hyprland.conf", "~/.config/hypr/hyprpaper.conf"]
reload = "hyprctl reload"
display_server = "wayland"
packages = ["hyprland"]

[kitty]
config_paths = ["~/.config/kitty/kitty.conf"]
reload = ""
hot_reload = true
display_server = "both"
packages = ["kitty"]
```

---

## Supported Components in v0.2

### Window Managers (X11)
- bspwm       → ~/.config/bspwm/bspwmrc         → reload: bspc wm -r
- i3          → ~/.config/i3/config              → reload: i3-msg restart
- openbox     → ~/.config/openbox/rc.xml         → reload: openbox --restart
- awesome     → ~/.config/awesome/rc.lua         → reload: echo 'awesome.restart()' | awesome-client
- herbstluftwm → ~/.config/herbstluftwm/autostart → reload: herbstclient reload

### Wayland Compositors
- hyprland    → ~/.config/hypr/hyprland.conf     → reload: hyprctl reload
- sway        → ~/.config/sway/config            → reload: swaymsg reload
- niri        → ~/.config/niri/config.kdl        → reload: niri msg action do-screen-transition
- river       → ~/.config/river/init             → reload: riverctl spawn init

### Bars
- polybar     → ~/.config/polybar/config         → reload: killall -q polybar; polybar &        (X11)
- waybar      → ~/.config/waybar/config + style.css → reload: killall -q waybar; waybar &      (Wayland)
- eww         → ~/.config/eww/eww.yuck           → reload: eww reload                          (Both)

### Launchers (stateless, no reload needed)
- rofi        → ~/.config/rofi/config.rasi
- wofi        → ~/.config/wofi/config
- tofi        → ~/.config/tofi/config
- fuzzel      → ~/.config/fuzzel/fuzzel.ini

### Terminals (all auto hot-reload, no reload command needed)
- kitty       → ~/.config/kitty/kitty.conf
- alacritty   → ~/.config/alacritty/alacritty.toml
- wezterm     → ~/.config/wezterm/wezterm.lua
- foot        → ~/.config/foot/foot.ini

### Notifications
- dunst       → ~/.config/dunst/dunstrc          → reload: killall -q dunst; dunst &           (X11)
- mako        → ~/.config/mako/config            → reload: makoctl reload                      (Wayland)
- swaync      → ~/.config/swaync/config.json     → reload: swaync-client --reload              (Wayland)

### Compositors (X11)
- picom       → ~/.config/picom/picom.conf       → reload: pkill -q picom; picom --daemon

### Wallpaper Setters
- feh         → stateless                        → reload: feh --bg-scale <wallpaper_path>     (X11)
- nitrogen    → ~/.config/nitrogen/              → reload: nitrogen --restore                  (X11)
- hyprpaper   → ~/.config/hypr/hyprpaper.conf    → reload: hyprctl hyprpaper reload            (Wayland)
- swww        → stateless                        → reload: swww img <wallpaper_path>           (Wayland)
- swaybg      → stateless                        → reload: pkill swaybg; swaybg -i <path> &   (Wayland)

### Lock Screens
- i3lock      → stateless                        (X11)
- swaylock    → ~/.config/swaylock/config        (Wayland)
- hyprlock    → ~/.config/hypr/hyprlock.conf     (Wayland)

### Shell Prompts
- starship    → ~/.config/starship.toml          → stateless, reloads per instance

---

## Dependency Handling

rixi detects the distro via /etc/os-release and maps to the correct package manager.

Supported distros and package managers:
- Arch / Manjaro / EndeavourOS   → pacman (AUR via yay if available)
- Fedora / RHEL / CentOS         → dnf
- Debian / Ubuntu / Mint / Pop   → apt
- openSUSE                       → zypper
- Void Linux                     → xbps-install
- NixOS                          → nix-env
- Unknown                        → print raw package names, no command

### Dependency check flow

Before applying any rice:

1. Read the manifest [dependencies] section
2. Check each declared package via `which` or distro-specific checks
3. Check fonts via `fc-list | grep -i "<fontname>"`
4. Print a dependency report

### Dependency report output format

```
Checking dependencies...
  ✓ hyprland     installed
  ✓ kitty        installed
  ✗ waybar       missing
  ✗ rofi         missing
  ✗ swww         missing

  ✗ JetBrainsMono Nerd Font    not found (fc-list check failed)
  ✗ Papirus icons              not found

Detected distro: Arch Linux

Run this to install missing packages:
  sudo pacman -S waybar rofi swww

Manual installs required:
  Fonts    → https://nerdfonts.com (install JetBrainsMono Nerd Font)
  Icons    → sudo pacman -S papirus-icon-theme

Continue applying without missing dependencies? [y/N]
```

rixi never auto-installs anything. It prints the exact command to run.
If the user types N, apply is cancelled. If Y, apply proceeds with a warning
that some components may not render correctly.

---

## Shell Configuration

Shell configs are NOT treated like regular component configs.
rixi NEVER overwrites .zshrc, .bashrc, or config.fish directly.
These files contain personal aliases, PATH exports, and tool initialization
that have nothing to do with ricing.

rixi only manages the aesthetic layer of shell configuration:
- The prompt (starship, p10k)
- Color sourcing (pywal, theme colors)

### zsh and bash — managed block approach

On first `rixi apply`, rixi appends ONE line to the end of ~/.zshrc (or ~/.bashrc):

```bash
# rixi shell theme — do not remove
source ~/.local/share/rixi/shell.sh
```

This line is added only once and never modified again.

rixi owns ~/.local/share/rixi/shell.sh entirely.
When a rice is applied, rixi rewrites only shell.sh:

```bash
# managed by rixi — current rice: sathiya/gruvbox
# do not edit manually
eval "$(starship init zsh)"
```

On rollback, rixi reverts shell.sh to the previous snapshot version.
The source line in .zshrc/bashrc stays but sources the rolled-back shell.sh.

### fish — conf.d approach

Fish automatically sources all .fish files in ~/.config/fish/conf.d/
rixi drops exactly one file there:

  ~/.config/fish/conf.d/rixi.fish

rixi owns rixi.fish entirely. config.fish is never touched.

When a rice is applied, rixi rewrites rixi.fish:

```fish
# managed by rixi — current rice: sathiya/gruvbox
# do not edit manually
starship init fish | source
```

On rollback, rixi reverts rixi.fish to the previous snapshot version.

### Shell component in manifest

```toml
[meta]
...
components = ["hyprland", "waybar", "rofi", "kitty", "starship"]

[shell]
type = "zsh"          # zsh | bash | fish
prompt = "starship"   # starship | p10k | oh-my-zsh | none
```

If no [shell] section is present, rixi skips shell configuration entirely.

---

## Updated Manifest Format for v0.2

```toml
[meta]
name = "gruvbox"
author = "sathiya"
version = "0.2.0"
wm = "bspwm"
display_server = "x11"
colorscheme = "gruvbox"
components = ["bspwm", "polybar", "rofi", "dunst", "kitty", "picom", "starship"]
tags = ["minimal", "dark", "gruvbox"]
description = "minimal gruvbox bspwm setup"

[dependencies]
packages = ["bspwm", "sxhkd", "polybar", "rofi", "dunst", "kitty", "picom"]
fonts = ["JetBrainsMono Nerd Font"]
icons = ["Papirus"]

[shell]
type = "zsh"
prompt = "starship"

[wallpaper]
file = "walls/gruvbox.png"
setter = "feh"

[overrides]
# only for non-standard config paths
# "polybar" = "~/.config/polybar/custom/config"

[hooks]
post_apply = []    # declared but NOT executed in v0.2, reserved for v0.3
```

---

## Updated rixi apply flow for v0.2

1. Parse and validate manifest
2. Fetch/verify external component index (components.toml)
3. Run dependency check — print report
4. Prompt user to continue if dependencies missing
5. Snapshot current state to ~/.local/share/rixi/snapshots/<timestamp>/
   - Snapshot includes shell.sh / rixi.fish if shell component present
6. Apply component config files per component index paths
7. Handle shell configuration:
   - zsh/bash: inject source line if not present, rewrite shell.sh
   - fish: rewrite rixi.fish in conf.d/
8. Set wallpaper via detected setter
9. Reload components per index reload commands
10. Update state.toml
11. Print apply summary

---

## v0.2 Definition of Done

- All components in the supported list above are recognized and applied correctly
- Dependency check runs before every apply with distro-aware output
- Shell configuration is handled safely — .zshrc/.bashrc never overwritten
- fish conf.d approach works
- Font detection via fc-list works
- Wallpaper setters (feh, swww, hyprpaper, nitrogen, swaybg) all work
- External component index is fetched and cached
- Snapshots include shell config files for clean rollback

---

## What v0.2 Deliberately Ignores

- No Landlock sandboxing (v0.3)
- No static analysis of hooks (v0.3)
- No registry / rixi push / rixi pull (v0.4)
- No TUI (v0.5)
- No font auto-install
- No hook execution (hooks are parsed, stored, ignored)