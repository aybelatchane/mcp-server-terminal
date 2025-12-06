# MCP Registry Submissions

## Docker Hub

**Status**: ✅ Published

- **Image**: `abelatchane/mcp-server-terminal`
- **Tags**: `latest`, `1.0.0`
- **URL**: https://hub.docker.com/r/abelatchane/mcp-server-terminal

---

## Smithery

**Status**: 🔧 Ready to Submit

Smithery is the largest MCP registry with 4000+ servers.

### Submission Steps:

1. Push `smithery.yaml` to main branch:
   ```bash
   gh auth login
   git push origin dev
   # Create PR to merge to main
   ```

2. Visit https://smithery.ai/submit

3. Enter repository URL: `https://github.com/aybelatchane/mcp-server-terminal`

4. Smithery will automatically detect `smithery.yaml` and publish

### Configuration File: `smithery.yaml`
```yaml
startCommand:
  type: stdio
  configSchema:
    type: object
    properties:
      headless:
        type: boolean
        description: Run in headless mode without visual terminal windows
        default: false
  commandFunction: |
    (config) => ({
      command: "npx",
      args: config.headless
        ? ["-y", "mcp-server-terminal", "--headless"]
        : ["-y", "mcp-server-terminal"],
      env: { DISPLAY: ":0" }
    })
```

---

## mcp.so

**Status**: 📝 Ready to Submit

mcp.so is a community-maintained registry.

### Submission Steps:

1. Open an issue at: https://github.com/punkpeye/awesome-mcp-servers/issues/new

2. Use this template:

**Title**: `Add terminal-mcp - Terminal automation for AI agents`

**Body**:
```markdown
## Server Name
terminal-mcp (mcp-server-terminal)

## Repository
https://github.com/aybelatchane/mcp-server-terminal

## npm Package
https://www.npmjs.com/package/mcp-server-terminal

## Description
MCP server enabling AI agents to interact with terminal applications through structured Terminal State Tree representation. Think of it as "Playwright for terminals" - providing structured, queryable terminal state equivalent to Playwright's accessibility tree for web applications.

## Features
- 10 MCP tools for terminal interaction
- Terminal State Tree with UI element detection
- Visual mode (xterm windows) or headless operation
- Cross-platform: Linux, macOS, Windows (WSL)

## Installation
```bash
npx mcp-server-terminal
```

## Category
Utilities / Automation
```

---

## Glama

**Status**: 📝 Ready to Submit

Glama is an MCP registry and client platform.

### Submission Steps:

1. Visit https://glama.ai/mcp/servers

2. Click "Add Server" button

3. Fill in the form:
   - **Name**: Terminal MCP Server
   - **Repository**: https://github.com/aybelatchane/mcp-server-terminal
   - **NPM Package**: mcp-server-terminal
   - **Description**: MCP server for AI agents to interact with terminal applications through structured Terminal State Tree representation
   - **Category**: Utilities

---

## Additional Registries (Optional)

### MCP Hub (mcphub.io)
- URL: https://mcphub.io
- Submit via their web form

### Model Context Protocol Official
- URL: https://modelcontextprotocol.io/servers
- Submit via GitHub PR to Anthropic's MCP documentation

---

## Notes

- All submissions require the npm package to be published (✅ Done)
- Some registries may take 1-7 days to review and approve
- After approval, the server will appear in registry search results
