# easygit

easygit is a keyboard-first Git TUI built with Ratatui, Crossterm, and gix. Browse branches, commits, stashes, and details without leaving your terminal.

## Features
- Branch, commit, stash, and details panels navigated via hotkeys
- Branch creation, checkout, and deletion directly from the TUI
- Keyboard-driven workflow; no mouse required

## Install
With Rust toolchain:
```bash
cargo install easygit
easygit
```

Don’t have cargo installed? Follow the official Rust install guide: https://www.rust-lang.org/tools/install

## Usage
- Launch: `easygit`
- Global keys: `b` branches, `c` commits, `d` details, `s` stashes, `q` quit
- Branch panel: `↑/↓` move hover, `Enter` checkout, `a` add branch, `x`/`Delete` delete

## License
MIT
