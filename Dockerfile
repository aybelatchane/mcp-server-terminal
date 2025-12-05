# Multi-stage build for minimal final image
FROM rust:1.85-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build the release binary
RUN cargo build --release --bin terminal-mcp

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tmux \
    xterm \
    bash \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 mcp

# Copy binary from builder
COPY --from=builder /app/target/release/terminal-mcp /usr/local/bin/terminal-mcp

# Set ownership
RUN chown mcp:mcp /usr/local/bin/terminal-mcp

# Switch to non-root user
USER mcp
WORKDIR /home/mcp

# Expose MCP protocol on stdio (no ports needed)
# MCP uses stdio transport by default

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/terminal-mcp"]

# Health check (optional, for monitoring)
HEALTHCHECK NONE

# Labels for metadata
LABEL org.opencontainers.image.title="Terminal MCP Server"
LABEL org.opencontainers.image.description="Model Context Protocol server for terminal-based applications"
LABEL org.opencontainers.image.authors="Ayoub Belayoub"
LABEL org.opencontainers.image.url="https://github.com/aybelatchane/mcp-server-terminal"
LABEL org.opencontainers.image.source="https://github.com/aybelatchane/mcp-server-terminal"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.version="1.0.0"
