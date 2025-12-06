# easygit

easygit is a keyboard-first Git TUI built with Ratatui, Crossterm, and gix. Browse branches, commits, stashes, and details without leaving your terminal.

## Features
- Keyboard-driven Git TUI with branch, commit, stash, and details panels
- Branch management: create, checkout, and delete branches from the UI
- Commit workflow: stage/unstage selected files and create commits without leaving the terminal
- Navigate commits and stashes with hotkeys; quick panel switching (`b/c/s/d`, `q` to quit)
- Built on Ratatui + Crossterm for a responsive terminal layout

## Install
With Rust toolchain:
```bash
cargo install easygit
easygit
```

Don’t have cargo installed? Follow the official Rust install guide: https://www.rust-lang.org/tools/install

## License
MIT
