#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const { spawn } = require('node:child_process');

const binaryName = process.platform === 'win32' ? 'easygit.exe' : 'easygit';
const binaryPath = path.join(__dirname, binaryName);

if (!fs.existsSync(binaryPath)) {
  console.error(`easygit binary not found at ${binaryPath}.`);
  console.error('Try reinstalling the package or set EASYGIT_BINARY_HOST to a reachable release URL.');
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), { stdio: 'inherit' });

child.on('exit', (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code === undefined ? 0 : code);
  }
});

child.on('error', (err) => {
  console.error('Failed to launch easygit binary:', err);
  process.exit(1);
});
