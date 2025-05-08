# ---- Build Stage ----
FROM rust:latest as builder

# Create app directory
WORKDIR /usr/src/app

# Cache dependencies first (better layer caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -r src

# Copy source code
COPY . .

# Build the actual app
RUN cargo build --release

# ---- Runtime Stage ----
FROM ubuntu:latest

# Install necessary system dependencies (if needed, e.g., openssl)
RUN apt-get update

# Create non-root user
RUN useradd -m appuser

# Copy the binary from the builder
COPY --from=builder /usr/src/app/target/release/teoneo-courses /usr/local/bin/app

# Change to non-root user
USER appuser

# Run the app
ENTRYPOINT ["/usr/local/bin/app"]
