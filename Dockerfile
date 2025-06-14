# Multi-stage build for optimized image size
FROM rust:1.82-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml ./
COPY Cargo.lock* ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless dependencies change)
RUN cargo build --release
RUN rm src/main.rs

# Copy source code
COPY . .

# Build the application
# Touch main.rs to ensure it's rebuilt
RUN touch src/main.rs
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -r -s /bin/false appuser

# Create app directory
WORKDIR /app

# Copy the built binary from builder stage
COPY --from=builder /app/target/release/engine /app/engine

# Change ownership to app user
RUN chown appuser:appuser /app/engine

# Switch to app user
USER appuser

# Expose port (default 3000, configurable via PORT env var)
EXPOSE 3000

# Health check  
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:${PORT:-3000}/health || exit 1

# Set default environment variables for feature flags (can be overridden)
ENV FF_ENV_ID=""
ENV FF_AGENT_ID=""
ENV FF_PROJECT_ID=""
ENV PORT=3000

# Run the binary
CMD ["./engine"]