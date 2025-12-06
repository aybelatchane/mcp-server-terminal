# Changelog

All notable changes to Terminal MCP Server will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2025-12-06

### Added
- **Auto-update mechanism** - CLI automatically checks for and downloads newer versions from GitHub releases
- **Background color detection** - Menu detector now recognizes selections styled with background colors (e.g., Bubble Tea TUIs)
- **Version caching** - Binary cached in `~/.cache/mcp-server-terminal/` persists across npx runs

### Fixed
- **ANSI escape sequence preservation** - tmux capture-pane now uses `-e` flag to preserve color/style codes
- **Private mode sequence handling** - VTE parser now handles alternate screen buffer sequences (`\x1b[?1049h`)

### Changed
- **Cache location** - Moved from package directory to user home for persistence across npx updates

## [1.0.0] - 2025-12-02

### 🎉 Initial Production Release

Terminal MCP Server v1.0.0 is production-ready with complete feature set, comprehensive testing, and zero clippy warnings.

### Added

#### Core Features
- **10 MCP Tools** for complete terminal interaction
- **8 UI Element Detectors** (borders, menus, tables, inputs, buttons, checkboxes, progress, status bars)
- **Visual Terminal Mode** with popup windows
- **Session Recording** in asciinema v2 format

#### Performance
- **53.6µs snapshot latency** (930× faster than 50ms target)
- **Zero unsafe code** - Memory-safe Rust
- **242 tests passing** with 100% pass rate

[1.0.1]: https://github.com/aybelatchane/mcp-server-terminal/releases/tag/v1.0.1
[1.0.0]: https://github.com/aybelatchane/mcp-server-terminal/releases/tag/v1.0.0
