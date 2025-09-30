# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY locales ./locales
COPY templates ./templates
COPY static ./static

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
        ca-certificates \
        libssl3 \
        && \
    rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m devicl

# Create directories
RUN mkdir -p /app/data && \
    chown -R devicl:devicl /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/devicl /usr/local/bin/devicl

# Copy static assets
COPY --from=builder /app/locales /app/locales
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static

# Set permissions
RUN chown -R devicl:devicl /app

# Switch to app user
USER devicl

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/api/auth/status || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:///app/data/devicl.db

# Run the application
CMD ["/usr/local/bin/devicl"]