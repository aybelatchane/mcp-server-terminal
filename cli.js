#!/usr/bin/env node

const https = require("https");
const fs = require("fs");
const path = require("path");
const { execSync, spawn } = require("child_process");

const PACKAGE_VERSION = require("./package.json").version;
const REPO = "aybelatchane/mcp-server-terminal";
const BINARY_NAME = "terminal-mcp";

// Cache directory for the binary (in user's home to persist across npx runs)
const CACHE_DIR = path.join(
  process.env.HOME || process.env.USERPROFILE || __dirname,
  ".cache",
  "mcp-server-terminal"
);
const BINARY_PATH = path.join(CACHE_DIR, BINARY_NAME);
const VERSION_FILE = path.join(CACHE_DIR, "version.txt");

// Map Node.js platform/arch to Rust target triples
// Linux uses musl for fully static binaries (works on all distros regardless of glibc version)
const PLATFORM_MAPPING = {
  darwin: {
    x64: "x86_64-apple-darwin",
    arm64: "aarch64-apple-darwin",
  },
  linux: {
    x64: "x86_64-unknown-linux-musl",
    arm64: "aarch64-unknown-linux-musl",
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

function getDownloadUrl(version, target) {
  return `https://github.com/${REPO}/releases/download/v${version}/${BINARY_NAME}-${target}.tar.gz`;
}

// Fetch latest version from GitHub releases
function fetchLatestVersion() {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: "api.github.com",
      path: `/repos/${REPO}/releases/latest`,
      headers: {
        "User-Agent": "mcp-server-terminal-cli",
        Accept: "application/vnd.github.v3+json",
      },
    };

    https
      .get(options, (response) => {
        if (response.statusCode === 404) {
          // No releases yet, use package version
          resolve(PACKAGE_VERSION);
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`GitHub API error: ${response.statusCode}`));
          return;
        }

        let data = "";
        response.on("data", (chunk) => (data += chunk));
        response.on("end", () => {
          try {
            const release = JSON.parse(data);
            // tag_name is like "v1.0.1", strip the "v"
            const version = release.tag_name.replace(/^v/, "");
            resolve(version);
          } catch (e) {
            reject(e);
          }
        });
      })
      .on("error", reject);
  });
}

// Get cached version
function getCachedVersion() {
  try {
    if (fs.existsSync(VERSION_FILE)) {
      return fs.readFileSync(VERSION_FILE, "utf8").trim();
    }
  } catch {
    // Ignore errors
  }
  return null;
}

// Save version to cache
function saveCachedVersion(version) {
  try {
    fs.writeFileSync(VERSION_FILE, version);
  } catch {
    // Ignore errors
  }
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

// Compare versions (simple semver comparison)
function isNewerVersion(latest, current) {
  const latestParts = latest.split(".").map(Number);
  const currentParts = current.split(".").map(Number);

  for (let i = 0; i < 3; i++) {
    const l = latestParts[i] || 0;
    const c = currentParts[i] || 0;
    if (l > c) return true;
    if (l < c) return false;
  }
  return false;
}

async function ensureBinary() {
  // Create cache directory if needed
  if (!fs.existsSync(CACHE_DIR)) {
    fs.mkdirSync(CACHE_DIR, { recursive: true });
  }

  const cachedVersion = getCachedVersion();
  let targetVersion = PACKAGE_VERSION;
  let needsDownload = !fs.existsSync(BINARY_PATH);

  // Check for updates (unless MCP_NO_UPDATE is set)
  if (!process.env.MCP_NO_UPDATE) {
    try {
      const latestVersion = await fetchLatestVersion();

      if (cachedVersion && isNewerVersion(latestVersion, cachedVersion)) {
        console.error(
          `Update available: v${cachedVersion} -> v${latestVersion}`
        );
        targetVersion = latestVersion;
        needsDownload = true;
      } else if (!cachedVersion) {
        // No cached version, use latest
        targetVersion = latestVersion;
      }
    } catch (error) {
      // Failed to check for updates, continue with cached or package version
      if (process.env.DEBUG) {
        console.error(`Update check failed: ${error.message}`);
      }
    }
  }

  // If binary exists and version matches, verify it's executable
  if (!needsDownload && fs.existsSync(BINARY_PATH)) {
    try {
      fs.accessSync(BINARY_PATH, fs.constants.X_OK);
      return BINARY_PATH;
    } catch {
      needsDownload = true;
    }
  }

  if (!needsDownload) {
    return BINARY_PATH;
  }

  const target = getTarget();
  const url = getDownloadUrl(targetVersion, target);

  console.error(
    `Downloading mcp-server-terminal v${targetVersion} for ${target}...`
  );

  try {
    const buffer = await downloadFile(url);
    extractTarGz(buffer, CACHE_DIR);
    fs.chmodSync(BINARY_PATH, 0o755);
    saveCachedVersion(targetVersion);
    console.error(`Successfully installed mcp-server-terminal v${targetVersion}`);
    return BINARY_PATH;
  } catch (error) {
    console.error(`Failed to download binary: ${error.message}`);
    console.error(`\nPlease download manually from:`);
    console.error(`https://github.com/${REPO}/releases/tag/v${targetVersion}`);
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
