# ============================
# 1. BUILDER STAGE
# ============================
FROM rustlang/rust:nightly as builder

WORKDIR /app

# Copy full source code (required for complete build)
COPY . .

# Build in release mode
RUN cargo build --release

# ============================
# 2. RUNTIME STAGE
# ============================
FROM debian:stable-slim AS runtime

# Install minimal libs needed by Rust binaries
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    && rm -rf /var/lib/apt/lists/*

# Create runtime directory for AegFileSystem
RUN mkdir -p /etc/aegisr/config
RUN mkdir -p /etc/aegisr/logs

# Bring the built binary from builder stage
COPY --from=builder /app/target/release/aegisr-daemon /usr/local/bin/aegisr-daemon

# Expose your daemon port
EXPOSE 1211

# Optional: Set environment defaults
ENV AEGISR_LOG_LEVEL=info

# Run daemon with default config or allow CLI override via docker run
ENTRYPOINT ["/usr/local/bin/aegisr-daemon"]
CMD ["--host", "0.0.0.0", "--port", "1211"]
