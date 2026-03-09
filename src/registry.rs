use std::collections::HashMap;

/// A component in the built-in registry — maps a component name
/// to its standard XDG config file paths.
#[derive(Debug, Clone)]
pub struct ComponentEntry {
    pub paths: Vec<&'static str>,
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
        },
    );

    map.insert(
        "polybar",
        ComponentEntry {
            paths: vec![
                "~/.config/polybar/config",
                "~/.config/polybar/colors.ini",
            ],
        },
    );

    map.insert(
        "rofi",
        ComponentEntry {
            paths: vec!["~/.config/rofi/config.rasi"],
        },
    );

    map.insert(
        "dunst",
        ComponentEntry {
            paths: vec!["~/.config/dunst/dunstrc"],
        },
    );

    map.insert(
        "hyprland",
        ComponentEntry {
            paths: vec![
                "~/.config/hypr/hyprland.conf",
                "~/.config/hypr/hyprpaper.conf",
            ],
        },
    );

    map.insert(
        "kitty",
        ComponentEntry {
            paths: vec!["~/.config/kitty/kitty.conf"],
        },
    );

    map.insert(
        "alacritty",
        ComponentEntry {
            paths: vec!["~/.config/alacritty/alacritty.toml"],
        },
    );

    map.insert(
        "picom",
        ComponentEntry {
            paths: vec!["~/.config/picom/picom.conf"],
        },
    );

    map.insert(
        "sway",
        ComponentEntry {
            paths: vec!["~/.config/sway/config"],
        },
    );

    map.insert(
        "waybar",
        ComponentEntry {
            paths: vec![
                "~/.config/waybar/config",
                "~/.config/waybar/style.css",
            ],
        },
    );

    map
}
