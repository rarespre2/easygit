# easygit (npm wrapper)

This package bundles a small launcher and installs the prebuilt `easygit` binary for your platform during `npm install`.

## Install
```bash
npx easygit
# or
npm i -g easygit
easygit
```

## How it works
- The `postinstall` script downloads a platform-specific archive from the GitHub release that matches the npm package version (tagged `v<version>`).
- Archives must be named `easygit-<platform>-<arch>.tar.gz` for Linux/macOS and `easygit-<platform>-<arch>.zip` for Windows, containing the `easygit` binary at the archive root.
- Supported `platform` values: `linux`, `macos`, `windows`; `arch` values: `x64`, `arm64`.

### Environment variables
- `EASYGIT_BINARY_HOST`: Override the download host (defaults to `https://github.com/rarespredoi/easygit/releases/download`).
- `EASYGIT_SKIP_POSTINSTALL=1`: Skip the binary download (only for development).
