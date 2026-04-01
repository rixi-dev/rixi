# Contributing to RIXI

Here's how to contribute the two things that matter most, themes and components. Plus a quick note on code changes.

---

## Submitting a Theme

Make sure your rice is set up locally first:

```bash
rixi init
```

Your rice directory needs three things before you push:

```
~/.local/share/rixi/store/<author>/<theme>/
  manifest.toml
  configs/          ← at least one component directory
  preview.png
```

Then push it to the community registry:

```bash
rixi push author/theme
```

This opens a PR against `rixi-dev/themes` directly from your terminal.

To verify your rice pulls correctly before submitting:

```bash
rixi pull author/theme
```

### GitHub Authorization

The first time you run `rixi push`, RIXI will open a browser window and ask you to enter a code:
```
Opened browser for authorization. Enter code: XXXX-XXXX
```

Enter the code and authorize. This only happens once — RIXI stores the token locally after that.
---

## Adding a Component

Components live in the built-in registry inside `src/`. Each entry maps a tool name to its config path on the system.

To add support for a new tool:

1. Find the registry in `src/` (look for where existing components like `bspwm` or `alacritty` are defined)
2. Add your component.(tool name, config path, and category)
3. Test that `rixi init` detects it and `rixi apply` copies it correctly
4. Open a PR with the component name in the title and a note on what distros/setups you tested on

---

## Code Changes

- Run `cargo fmt` and `cargo clippy` before opening a PR
- Keep PRs focused — one thing per PR
- If you're changing command behavior or output, update the README too
- For anything large, open an issue first

---

## Bugs

Include your distro, the exact command, what you expected, and what actually happened.