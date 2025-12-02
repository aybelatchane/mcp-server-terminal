#!/usr/bin/env node

const https = require("https");
const fs = require("fs");
const path = require("path");
const { execSync, spawn } = require("child_process");

const PACKAGE_VERSION = require("./package.json").version;
const REPO = "aybelatchane/mcp-server-terminal";
const BINARY_NAME = "terminal-mcp";

// Cache directory for the binary
const CACHE_DIR = path.join(__dirname, ".cache");
const BINARY_PATH = path.join(CACHE_DIR, BINARY_NAME);

// Map Node.js platform/arch to Rust target triples
const PLATFORM_MAPPING = {
  darwin: {
    x64: "x86_64-apple-darwin",
    arm64: "aarch64-apple-darwin",
  },
  linux: {
    x64: "x86_64-unknown-linux-gnu",
    arm64: "aarch64-unknown-linux-gnu",
  },
};

function getTarget() {
  const platform = process.platform;
  const arch = process.arch;

  if (!PLATFORM_MAPPING[platform]) {
    console.error(`Unsupported platform: ${platform}`);
    console.error("Supported platforms: macOS (darwin), Linux");
    process.exit(1);
  }

  const target = PLATFORM_MAPPING[platform][arch];
  if (!target) {
    console.error(`Unsupported architecture: ${arch} on ${platform}`);
    console.error("Supported architectures: x64, arm64");
    process.exit(1);
  }

  return target;
}

function getDownloadUrl(target) {
  return `https://github.com/${REPO}/releases/download/v${PACKAGE_VERSION}/${BINARY_NAME}-${target}.tar.gz`;
}

function downloadFile(url) {
  return new Promise((resolve, reject) => {
    const request = (url) => {
      https
        .get(url, (response) => {
          if (response.statusCode === 302 || response.statusCode === 301) {
            request(response.headers.location);
            return;
          }

          if (response.statusCode !== 200) {
            reject(
              new Error(`Failed to download: HTTP ${response.statusCode}`)
            );
            return;
          }

          const chunks = [];
          response.on("data", (chunk) => chunks.push(chunk));
          response.on("end", () => resolve(Buffer.concat(chunks)));
          response.on("error", reject);
        })
        .on("error", reject);
    };
    request(url);
  });
}

function extractTarGz(buffer, destDir) {
  const tarballPath = path.join(destDir, "temp.tar.gz");
  fs.writeFileSync(tarballPath, buffer);

  try {
    execSync(`tar -xzf "${tarballPath}" -C "${destDir}"`, { stdio: "pipe" });
  } finally {
    fs.unlinkSync(tarballPath);
  }
}

async function ensureBinary() {
  // Check if binary already exists and is executable
  if (fs.existsSync(BINARY_PATH)) {
    try {
      fs.accessSync(BINARY_PATH, fs.constants.X_OK);
      return BINARY_PATH;
    } catch {
      // Binary exists but not executable, re-download
    }
  }

  // Create cache directory
  if (!fs.existsSync(CACHE_DIR)) {
    fs.mkdirSync(CACHE_DIR, { recursive: true });
  }

  const target = getTarget();
  const url = getDownloadUrl(target);

  console.error(`Downloading mcp-server-terminal v${PACKAGE_VERSION} for ${target}...`);

  try {
    const buffer = await downloadFile(url);
    extractTarGz(buffer, CACHE_DIR);
    fs.chmodSync(BINARY_PATH, 0o755);
    console.error(`Successfully installed mcp-server-terminal`);
    return BINARY_PATH;
  } catch (error) {
    console.error(`Failed to download binary: ${error.message}`);
    console.error(`\nPlease download manually from:`);
    console.error(`https://github.com/${REPO}/releases/tag/v${PACKAGE_VERSION}`);
    process.exit(1);
  }
}

async function main() {
  const binaryPath = await ensureBinary();

  // Pass all arguments to the binary
  const args = process.argv.slice(2);

  const child = spawn(binaryPath, args, {
    stdio: "inherit",
    env: process.env,
  });

  child.on("error", (error) => {
    console.error(`Failed to start: ${error.message}`);
    process.exit(1);
  });

  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code ?? 0);
    }
  });
}

main();
