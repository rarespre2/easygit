#!/usr/bin/env node
const { spawn } = require("child_process");
const fs = require("fs");
const path = require("path");

const binaryName = process.platform === "win32" ? "easygit.exe" : "easygit";
const candidates = [
  path.join(__dirname, "..", "bin", `${process.platform}-${process.arch}`, binaryName),
  path.join(__dirname, "..", "bin", binaryName)
];

const binPath = candidates.find((candidate) => fs.existsSync(candidate));

if (!binPath) {
  console.error(
    `easygit binary not found. Expected at: ${candidates.join(", ")}.\n` +
      "Build with `cargo build --release` and copy target binary into npm/bin/."
  );
  process.exit(1);
}

const child = spawn(binPath, process.argv.slice(2), {
  stdio: "inherit"
});

child.on("exit", (code) => {
  process.exit(code ?? 0);
});

child.on("error", (err) => {
  console.error("Failed to start easygit:", err);
  process.exit(1);
});
