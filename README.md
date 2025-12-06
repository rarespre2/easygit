# easygit

easygit is a Rust terminal UI for exploring Git concepts without leaving your shell. It uses Ratatui for layout/rendering and Crossterm for input/output, with gix powering Git interactions.

## Features
- Branch, commit, stash, and details panels navigated via hotkeys
- Branch creation, checkout, and deletion directly from the TUI
- Keyboard-driven workflow; no mouse required

## Controls
- Global: `b` branches, `c` commits, `d` details, `s` stashes, `q` quit
- Branch panel: `↑/↓` move hover, `Enter` checkout, `a` add branch, `x`/`Delete` delete

## Dependencies
- ratatui — terminal layout/rendering
- crossterm — cross-platform terminal events and drawing
- gix — Git operations (branches, checkout, delete, create)

## Getting started
```bash
cargo run
```

### Install via npm (MVP)
After we publish the npm package, you’ll be able to launch the TUI with:
```bash
npx easygit
# or install globally
npm i -g easygit
easygit
```

## Development
- Format and lint: `cargo fmt` and `cargo clippy --all-targets --all-features`
- Tests: `cargo test`
- Release build: `cargo build --release`

## License
MIT
