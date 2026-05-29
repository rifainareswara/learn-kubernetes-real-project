# =============================================================================
# Dockerfile — Rust Axum Backend (Multi-Stage Build)
# =============================================================================
#
# Multi-stage build menghasilkan image yang KECIL (~80-100MB)
# dibandingkan single-stage (~2GB+).
#
# Stage 1 (builder): Compile Rust binary
# Stage 2 (runtime): Copy binary ke image minimal
#
# Build:  docker build -t taskmanager-api .
# Run:    docker run -p 3000:3000 --env-file .env taskmanager-api
# =============================================================================

# ── Stage 1: Builder ─────────────────────────────────────────────────────────
# Pakai rust:slim (bukan rust:latest) untuk hemat download time.
FROM rust:slim-bookworm AS builder

# Install dependencies yang dibutuhkan untuk compile
# - pkg-config & libssl-dev: untuk crypto/TLS (meskipun kita pakai rustls,
#   beberapa crate masih butuh ini saat compile)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory di dalam container
WORKDIR /app

# ── Trick: Cache Dependencies ────────────────────────────────────────────────
# Copy Cargo.toml dan Cargo.lock DULU, lalu build dependencies.
# Ini membuat Docker cache layer dependencies, sehingga rebuild lebih cepat
# kalau hanya source code yang berubah (bukan dependencies).
COPY Cargo.toml Cargo.lock ./

# Buat dummy main.rs agar cargo build bisa compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies saja (akan di-cache oleh Docker)
RUN cargo build --release && rm -rf src

# ── Copy source code yang sebenarnya ─────────────────────────────────────────
COPY src/ src/
COPY migrations/ migrations/

# Touch main.rs agar cargo tahu file berubah dan perlu recompile
RUN touch src/main.rs

# Build binary final
RUN cargo build --release

# ── Stage 2: Runtime ─────────────────────────────────────────────────────────
# Pakai debian:bookworm-slim — image minimal (~80MB) yang compatible
# dengan binary yang di-compile di stage 1.
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies minimal
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Buat non-root user untuk keamanan
# (Jangan jalankan aplikasi sebagai root di production!)
RUN groupadd -r appuser && useradd -r -g appuser appuser

WORKDIR /app

# Copy HANYA binary dari builder stage (bukan source code!)
COPY --from=builder /app/target/release/taskmanager-api ./

# Copy migrations (dibutuhkan saat startup untuk auto-migrate)
COPY --from=builder /app/migrations ./migrations

# Ganti ke non-root user
USER appuser

# Expose port yang dipakai aplikasi
EXPOSE 3000

# Health check — Kubernetes juga punya probe, tapi ini backup
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["./taskmanager-api", "--health-check"] || exit 1

# Jalankan binary
CMD ["./taskmanager-api"]
