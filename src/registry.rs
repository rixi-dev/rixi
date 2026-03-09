use std::collections::HashMap;

/// A component in the built-in registry — maps a component name
/// to its standard XDG config file paths and an optional reload command.
#[derive(Debug, Clone)]
pub struct ComponentEntry {
    pub paths: Vec<&'static str>,
    /// Shell command to hot-reload the component after config changes.
    /// Empty string means the component reloads automatically (e.g. kitty).
    pub reload: &'static str,
}

/// Returns the built-in component registry.
/// This is the hardcoded map of known components and where their
/// config files live on disk.
pub fn builtin_registry() -> HashMap<&'static str, ComponentEntry> {
    let mut map = HashMap::new();

    map.insert(
        "bspwm",
        ComponentEntry {
            paths: vec!["~/.config/bspwm/bspwmrc"],
            reload: "bspc wm -r",
        },
    );

    map.insert(
        "polybar",
        ComponentEntry {
            paths: vec![
                "~/.config/polybar/config",
                "~/.config/polybar/colors.ini",
            ],
            reload: "killall -q polybar; polybar &",
        },
    );

    map.insert(
        "rofi",
        ComponentEntry {
            paths: vec!["~/.config/rofi/config.rasi"],
            reload: "",
        },
    );

    map.insert(
        "dunst",
        ComponentEntry {
            paths: vec!["~/.config/dunst/dunstrc"],
            reload: "killall -q dunst; dunst &",
        },
    );

    map.insert(
        "hyprland",
        ComponentEntry {
            paths: vec![
                "~/.config/hypr/hyprland.conf",
                "~/.config/hypr/hyprpaper.conf",
            ],
            reload: "hyprctl reload",
        },
    );

    map.insert(
        "kitty",
        ComponentEntry {
            paths: vec!["~/.config/kitty/kitty.conf"],
            reload: "",
        },
    );

    map.insert(
        "alacritty",
        ComponentEntry {
            paths: vec!["~/.config/alacritty/alacritty.toml"],
            reload: "",
        },
    );

    map.insert(
        "picom",
        ComponentEntry {
            paths: vec!["~/.config/picom/picom.conf"],
            reload: "pkill -q picom; picom --daemon",
        },
    );

    map.insert(
        "sway",
        ComponentEntry {
            paths: vec!["~/.config/sway/config"],
            reload: "swaymsg reload",
        },
    );

    map.insert(
        "waybar",
        ComponentEntry {
            paths: vec![
                "~/.config/waybar/config",
                "~/.config/waybar/style.css",
            ],
            reload: "killall -q waybar; waybar &",
        },
    );

    map
}
