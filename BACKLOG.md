# Development Backlog

This file contains the development backlog for Terminal MCP Server. These are internal development tasks not visible in the public repository.

---

## Epics

### [Epic] Package Manager Distribution and Release Automation
**Priority**: Critical
**Status**: Planning

Establish comprehensive package manager support to make Terminal MCP Server easily installable across all major platforms.

#### Scope
- Rust ecosystem (crates.io)
- macOS (Homebrew)
- Arch Linux (AUR)
- Debian/Ubuntu (apt)
- Automated publishing workflows

#### Related Tasks
- Publish to crates.io
- Create Homebrew formula
- Create AUR package
- Create Debian/Ubuntu package
- Unified Release Automation Workflow

---

### [Epic] Containerization & Portability Initiative
**Priority**: Medium
**Status**: Planning

Comprehensive containerization and portability initiative for Terminal MCP Server, making it production-ready for deployment across diverse environments.

#### Phases
1. **Foundation** (2 weeks): Multi-arch builds, CI/CD, security scanning
2. **Orchestration** (2 weeks): Kubernetes, Helm, cloud guides
3. **Advanced Features** (2 weeks): Monitoring, visual mode in containers
4. **Optimization** (2 weeks): Alpine image, performance tuning

#### Related Tasks
- Publish images to Docker Hub and GitHub Container Registry
- Create Alpine-based Docker image variant

---

## Packaging Tasks

### Publish to crates.io (Rust package registry)
**Priority**: Critical
**Effort**: 1 day

Terminal MCP is not available on crates.io. Users cannot install via `cargo install terminal-mcp`.

#### Acceptance Criteria
- [ ] Package published to crates.io as `terminal-mcp`
- [ ] Version matches git tag
- [ ] Package metadata complete
- [ ] Users can install via `cargo install terminal-mcp`
- [ ] Automated publishing on release

#### Implementation
```bash
# Dry run
cargo publish --dry-run

# Publish
cargo login
cargo publish
```

---

### Create Homebrew formula for macOS and Linux
**Priority**: Critical
**Effort**: 2 days

Make Terminal MCP available via Homebrew for macOS and Linux users.

#### Acceptance Criteria
- [ ] Homebrew formula created and tested
- [ ] Published to homebrew-tap repository
- [ ] Works on macOS (Intel + Apple Silicon)
- [ ] Works on Linux
- [ ] Users can install via `brew install aybelatchane/tap/terminal-mcp`
- [ ] Automated formula updates on release

#### Implementation
1. Create `aybelatchane/homebrew-tap` repository
2. Create `Formula/terminal-mcp.rb`
3. Test formula locally
4. Publish to tap repository

---

### Create AUR package for Arch Linux
**Priority**: Medium
**Effort**: 1-2 days

AUR package for Arch Linux users to install via yay/paru.

#### Acceptance Criteria
- [ ] PKGBUILD file created and tested
- [ ] Published to AUR as 'terminal-mcp'
- [ ] Users can install via AUR helpers
- [ ] Automated updates on release

---

### Create Debian/Ubuntu package (.deb)
**Priority**: Medium
**Effort**: 1 day

Create .deb package for Debian and Ubuntu users.

#### Acceptance Criteria
- [ ] .deb package buildable via cargo-deb
- [ ] Automated builds on release
- [ ] Published to GitHub releases
- [ ] Users can install via dpkg/apt

#### Installation
```bash
wget https://github.com/aybelatchane/mcp-server-terminal/releases/download/v1.0.0/terminal-mcp_1.0.0_amd64.deb
sudo dpkg -i terminal-mcp_1.0.0_amd64.deb
```

---

### Unified Release Automation Workflow
**Priority**: High
**Effort**: 2-3 days

Create a comprehensive GitHub Actions workflow that automates the entire release process across all distribution channels when a new version is tagged.

#### Trigger
```bash
git tag v1.0.1 && git push origin v1.0.1
```

#### Automated Steps
1. Validate version consistency
2. Publish to crates.io
3. Build and publish Docker images (multi-arch)
4. Update Homebrew formula
5. Update AUR PKGBUILD
6. Build Debian package
7. Create GitHub Release with changelog
8. Attach binary artifacts

---

## Containerization Tasks

### Publish images to Docker Hub and GitHub Container Registry
**Priority**: Critical
**Effort**: < 1 day

Establish dual publishing strategy to Docker Hub and GitHub Container Registry (GHCR).

#### Acceptance Criteria
- [ ] Official Docker Hub repository created (`aybelatchane/terminal-mcp`)
- [ ] Images published to `docker.io/aybelatchane/terminal-mcp`
- [ ] Images published to `ghcr.io/aybelatchane/terminal-mcp`
- [ ] Both registries have same tags

#### Pull Commands
```bash
# Docker Hub
docker pull aybelatchane/terminal-mcp:latest

# GitHub Container Registry
docker pull ghcr.io/aybelatchane/terminal-mcp:latest
```

---

### Create Alpine-based Docker image variant for smaller size
**Priority**: Medium
**Effort**: 2-3 days

Create an Alpine Linux-based Docker image variant to significantly reduce image size.

#### Acceptance Criteria
- [ ] Alpine-based Dockerfile (`Dockerfile.alpine`)
- [ ] Image size < 100MB (compressed)
- [ ] All functionality working (musl libc compatibility)
- [ ] Published as separate tag (`alpine`, `v1.0.0-alpine`)
- [ ] Performance benchmarks vs Debian-based image

#### Expected Results
| Metric | Debian (current) | Alpine (target) |
|--------|------------------|-----------------|
| Compressed size | ~200MB | < 100MB |
| Uncompressed | ~600MB | < 300MB |

---

## Completed (for reference)

These issues were completed before v1.0.0 release:
- Visual terminal mode implementation
- Session recording (Asciinema v2)
- All 10 MCP tools
- 8 UI element detectors
- Comprehensive logging
- npm package support
