use std::collections::HashMap;

/// A component in the built-in registry — maps a component name
/// to its standard XDG config file paths, reload command, and display server.
#[derive(Debug, Clone)]
pub struct ComponentEntry {
    pub paths: Vec<&'static str>,
    /// Shell command to hot-reload the component after config changes.
    /// Empty string means the component reloads automatically or is launched on demand.
    pub reload: &'static str,
    /// Display server: "x11", "wayland", or "both".
    pub display: &'static str,
}

/// Returns the built-in component registry.
pub fn builtin_registry() -> HashMap<&'static str, ComponentEntry> {
    let mut m = HashMap::new();

    // ── Window Managers (X11) ──────────────────────────────────────
    m.insert("bspwm", ComponentEntry {
        paths: vec!["~/.config/bspwm/bspwmrc"],
        reload: "bspc wm -r",
        display: "x11",
    });
    m.insert("i3", ComponentEntry {
        paths: vec!["~/.config/i3/config"],
        reload: "i3-msg restart",
        display: "x11",
    });
    m.insert("openbox", ComponentEntry {
        paths: vec!["~/.config/openbox/rc.xml"],
        reload: "openbox --restart",
        display: "x11",
    });
    m.insert("awesome", ComponentEntry {
        paths: vec!["~/.config/awesome/rc.lua"],
        reload: "echo 'awesome.restart()' | awesome-client",
        display: "x11",
    });
    m.insert("herbstluftwm", ComponentEntry {
        paths: vec!["~/.config/herbstluftwm/autostart"],
        reload: "herbstclient reload",
        display: "x11",
    });

    // ── Wayland Compositors ────────────────────────────────────────
    m.insert("hyprland", ComponentEntry {
        paths: vec!["~/.config/hypr/hyprland.conf"],
        reload: "hyprctl reload",
        display: "wayland",
    });
    m.insert("sway", ComponentEntry {
        paths: vec!["~/.config/sway/config"],
        reload: "swaymsg reload",
        display: "wayland",
    });
    m.insert("niri", ComponentEntry {
        paths: vec!["~/.config/niri/config.kdl"],
        reload: "niri msg action do-screen-transition",
        display: "wayland",
    });
    m.insert("river", ComponentEntry {
        paths: vec!["~/.config/river/init"],
        reload: "",
        display: "wayland",
    });

    // ── Bars ───────────────────────────────────────────────────────
    m.insert("polybar", ComponentEntry {
        paths: vec!["~/.config/polybar/config", "~/.config/polybar/colors.ini"],
        reload: "killall -q polybar; polybar &",
        display: "x11",
    });
    m.insert("waybar", ComponentEntry {
        paths: vec!["~/.config/waybar/config", "~/.config/waybar/style.css"],
        reload: "killall -q waybar; waybar &",
        display: "wayland",
    });
    m.insert("eww", ComponentEntry {
        paths: vec!["~/.config/eww/eww.yuck", "~/.config/eww/eww.scss"],
        reload: "eww reload",
        display: "both",
    });

    // ── Launchers ──────────────────────────────────────────────────
    m.insert("rofi", ComponentEntry {
        paths: vec!["~/.config/rofi/config.rasi"],
        reload: "",
        display: "x11",
    });
    m.insert("wofi", ComponentEntry {
        paths: vec!["~/.config/wofi/config"],
        reload: "",
        display: "wayland",
    });
    m.insert("tofi", ComponentEntry {
        paths: vec!["~/.config/tofi/config"],
        reload: "",
        display: "wayland",
    });
    m.insert("fuzzel", ComponentEntry {
        paths: vec!["~/.config/fuzzel/fuzzel.ini"],
        reload: "",
        display: "wayland",
    });

    // ── Terminals ──────────────────────────────────────────────────
    m.insert("kitty", ComponentEntry {
        paths: vec!["~/.config/kitty/kitty.conf"],
        reload: "",
        display: "both",
    });
    m.insert("alacritty", ComponentEntry {
        paths: vec!["~/.config/alacritty/alacritty.toml"],
        reload: "",
        display: "both",
    });
    m.insert("wezterm", ComponentEntry {
        paths: vec!["~/.config/wezterm/wezterm.lua"],
        reload: "",
        display: "both",
    });
    m.insert("foot", ComponentEntry {
        paths: vec!["~/.config/foot/foot.ini"],
        reload: "",
        display: "wayland",
    });

    // ── Notifications ──────────────────────────────────────────────
    m.insert("dunst", ComponentEntry {
        paths: vec!["~/.config/dunst/dunstrc"],
        reload: "killall -q dunst; dunst &",
        display: "x11",
    });
    m.insert("mako", ComponentEntry {
        paths: vec!["~/.config/mako/config"],
        reload: "makoctl reload",
        display: "wayland",
    });
    m.insert("swaync", ComponentEntry {
        paths: vec!["~/.config/swaync/config.json"],
        reload: "swaync-client --reload",
        display: "wayland",
    });

    // ── X11 Compositors ────────────────────────────────────────────
    m.insert("picom", ComponentEntry {
        paths: vec!["~/.config/picom/picom.conf"],
        reload: "pkill -q picom; picom --daemon",
        display: "x11",
    });

    // ── Wallpaper ──────────────────────────────────────────────────
    m.insert("hyprpaper", ComponentEntry {
        paths: vec!["~/.config/hypr/hyprpaper.conf"],
        reload: "hyprctl hyprpaper reload",
        display: "wayland",
    });
    m.insert("nitrogen", ComponentEntry {
        paths: vec!["~/.config/nitrogen/nitrogen.cfg"],
        reload: "nitrogen --restore",
        display: "x11",
    });

    // ── Lock Screens ───────────────────────────────────────────────
    m.insert("swaylock", ComponentEntry {
        paths: vec!["~/.config/swaylock/config"],
        reload: "",
        display: "wayland",
    });
    m.insert("hyprlock", ComponentEntry {
        paths: vec!["~/.config/hypr/hyprlock.conf"],
        reload: "",
        display: "wayland",
    });

    // ── Shell Prompts ──────────────────────────────────────────────
    m.insert("starship", ComponentEntry {
        paths: vec!["~/.config/starship.toml"],
        reload: "",
        display: "both",
    });

    m
}
