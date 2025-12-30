# Changelog

All notable changes to Terminal MCP Server will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.5] - 2025-12-30

### Fixed
- **CI/CD Windows build** - Added `shell: bash` to Windows build step to fix PowerShell syntax errors in release workflow
- **Tmux session persistence** - Added `remain-on-exit on` option to prevent "session no longer exists" errors when commands exit (#153)

### Changed
- **Linux binaries now use musl** - Switched from glibc to musl for fully static Linux binaries, fixing GLIBC version compatibility issues on older distributions (#152)

## [1.0.4] - 2025-12-30

### Fixed
- **GLIBC 2.39 dependency** - Linux binaries now statically linked with musl, works on Ubuntu 22.04 and other older distributions (#152)

## [1.0.3] - 2025-12-10

### Added
- **Native Windows support** - Full Windows x64 support without WSL dependency (#147)
  - Platform-specific process termination (`taskkill` on Windows, `SIGTERM` on Unix)
  - Windows visual mode via Windows Terminal or ConHost
  - ConPTY-based headless mode for full terminal emulation
  - Windows x64 added to CI test matrix and release builds

### Fixed
- **Gemini schema compatibility** - Schema transformer now removes `$schema` field to prevent draft version conflicts (#145)
  - Fixes: `no schema with key or ref "https://json-schema.org/draft/2020-12/schema"` error
  - Gemini now accepts tool schemas without draft version validation errors
  - Added `remove_schema_field()` transformation step
  - Added unit test for `$schema` field removal

## [1.0.2] - 2025-12-08

### Added
- **Working directory support** - `terminal_session_create` now accepts optional `cwd` parameter to start sessions in specific directories (#138)
- **JSON Schema transformation** - Automatic compatibility layer for AI clients requiring draft-07 schemas (Gemini support) (#129-134)
  - Transforms `$defs` to `definitions`
  - Simplifies nullable `anyOf` patterns
- **Integration tests** - Comprehensive test suite for detector false positive fixes (#141)

### Fixed
- **TUI menu rendering** - Menu detector now properly joins wrapped lines before detection, fixing menu parsing in interactive applications (#140)
- **Shell prompt false positives** - Git branch indicators like `(dev)`, `(main)` in shell prompts no longer detected as buttons (#141)
- **Random punctuation false positives** - Dots, dashes, and asterisks in regular text no longer incorrectly detected as progress bars (#141)
- **Buffer corruption in visual mode** - Fixed tmux pane content caching issues that caused snapshot corruption (#137)

### Changed
- **Button detector** - Removed parenthesis patterns `()` to prevent shell prompt false positives
- **Progress detector** - Now requires actual Unicode block characters (█▓▒░) to avoid matching ASCII punctuation

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

[1.0.5]: https://github.com/aybelatchane/mcp-server-terminal/releases/tag/v1.0.5
[1.0.4]: https://github.com/aybelatchane/mcp-server-terminal/releases/tag/v1.0.4
[1.0.3]: https://github.com/aybelatchane/mcp-server-terminal/releases/tag/v1.0.3
[1.0.2]: https://github.com/aybelatchane/mcp-server-terminal/releases/tag/v1.0.2
[1.0.1]: https://github.com/aybelatchane/mcp-server-terminal/releases/tag/v1.0.1
[1.0.0]: https://github.com/aybelatchane/mcp-server-terminal/releases/tag/v1.0.0
