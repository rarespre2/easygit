#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const https = require('node:https');
const { spawn } = require('node:child_process');

const pkg = require('../package.json');

const SKIP = process.env.EASYGIT_SKIP_POSTINSTALL === '1';
if (SKIP) {
  console.log('Skipping easygit binary download (EASYGIT_SKIP_POSTINSTALL=1).');
  process.exit(0);
}

const PLATFORM_MAP = {
  darwin: 'macos',
  linux: 'linux',
  win32: 'windows'
};

const ARCH_MAP = {
  x64: 'x64',
  arm64: 'arm64'
};

const platform = PLATFORM_MAP[process.platform];
const arch = ARCH_MAP[process.arch];

if (!platform || !arch) {
  console.error(`Unsupported platform/arch: ${process.platform} ${process.arch}.`);
  process.exit(1);
}

const tag = `v${pkg.version}`;
const binaryHost = process.env.EASYGIT_BINARY_HOST || 'https://github.com/rarespredoi/easygit/releases/download';
const isWindows = process.platform === 'win32';
const archiveName = `easygit-${platform}-${arch}${isWindows ? '.zip' : '.tar.gz'}`;
const downloadUrl = `${binaryHost}/${tag}/${archiveName}`;

const binDir = path.join(__dirname, '..', 'bin');
const archivePath = path.join(binDir, archiveName);
const binaryName = isWindows ? 'easygit.exe' : 'easygit';
const binaryPath = path.join(binDir, binaryName);

if (fs.existsSync(binaryPath)) {
  process.exit(0);
}

fs.mkdirSync(binDir, { recursive: true });

function fetch(url, dest, redirects = 0) {
  return new Promise((resolve, reject) => {
    const request = https.get(url, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        if (redirects > 5) {
          reject(new Error('Too many redirects while downloading binary.'));
          res.resume();
          return;
        }
        fetch(res.headers.location, dest, redirects + 1).then(resolve, reject);
        res.resume();
        return;
      }

      if (res.statusCode !== 200) {
        reject(new Error(`Download failed with status ${res.statusCode}`));
        res.resume();
        return;
      }

      const file = fs.createWriteStream(dest);
      res.pipe(file);
      file.on('finish', () => file.close(resolve));
      file.on('error', reject);
    });

    request.on('error', reject);
  });
}

function extract(archive, dest) {
  return new Promise((resolve, reject) => {
    if (isWindows) {
      const args = [
        '-NoLogo',
        '-NonInteractive',
        '-Command',
        `Expand-Archive -LiteralPath '${archive}' -DestinationPath '${dest}' -Force`
      ];
      const child = spawn('powershell', args, { stdio: 'inherit' });
      child.on('exit', (code) => (code === 0 ? resolve() : reject(new Error(`Expand-Archive exit code ${code}`))));
      child.on('error', reject);
      return;
    }

    const child = spawn('tar', ['-xzf', archive, '-C', dest], { stdio: 'inherit' });
    child.on('exit', (code) => (code === 0 ? resolve() : reject(new Error(`tar exit code ${code}`))));
    child.on('error', reject);
  });
}

function ensureExecutable(file) {
  if (isWindows) {
    return;
  }

  try {
    fs.chmodSync(file, 0o755);
  } catch (err) {
    console.warn(`Could not set executable bit on ${file}:`, err.message);
  }
}

(async () => {
  console.log(`Downloading easygit binary from ${downloadUrl}`);
  try {
    await fetch(downloadUrl, archivePath);
    await extract(archivePath, binDir);
    ensureExecutable(binaryPath);
    fs.rmSync(archivePath, { force: true });
    console.log('easygit binary installed.');
  } catch (err) {
    console.error('Failed to set up easygit binary.');
    console.error(err.message);
    console.error(`Manual fallback: download ${downloadUrl} and place ${binaryName} into ${binDir}`);
    process.exit(1);
  }
})();
