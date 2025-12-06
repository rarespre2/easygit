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

## Install via npm (prebuilt binary)
```bash
npx easygit
# or install globally
npm i -g easygit
easygit
```

The npm package downloads a prebuilt binary from the GitHub release matching the package version (`v<version>`). Supported platforms: Linux/macOS (x64, arm64) and Windows (x64, arm64).

### Publishing the npm package
1. Build and archive binaries per target:
   - Linux/macOS: place `easygit` at archive root and package as `easygit-<platform>-<arch>.tar.gz` (e.g., `easygit-linux-x64.tar.gz`, `easygit-macos-arm64.tar.gz`).
   - Windows: place `easygit.exe` at archive root and package as `easygit-windows-<arch>.zip` (e.g., `easygit-windows-x64.zip`).
2. Create a GitHub release tagged `v<version>` and upload those archives.
3. From `npm/`, run `npm publish` (the `postinstall` script will fetch the correct archive for end users). Use `EASYGIT_BINARY_HOST` to point at a different host if needed.

## Development
- Format and lint: `cargo fmt` and `cargo clippy --all-targets --all-features`
- Tests: `cargo test`
- Release build: `cargo build --release`

## License
MIT
