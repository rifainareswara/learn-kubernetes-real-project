// =============================================================================
// main.rs — Application Entry Point
// =============================================================================
//
// File ini adalah titik masuk aplikasi. Tugasnya:
//   1. Load environment variables dari .env
//   2. Inisialisasi logging
//   3. Load konfigurasi
//   4. Koneksi ke database
//   5. Jalankan migrations
//   6. Build aplikasi (routes + middleware)
//   7. Jalankan HTTP server
//
// Semua logika bisnis ada di lib.rs dan module-module lainnya.
// main.rs hanya melakukan "wiring" (menghubungkan semuanya).
//
// ┌─────────────────────────────────────────────────────────┐
// │                  STARTUP SEQUENCE                        │
// │                                                          │
// │  1. Load .env              ← dotenvy                    │
// │  2. Init logging           ← tracing_subscriber         │
// │  3. Config::from_env()     ← baca env vars              │
// │  4. create_pool()          ← koneksi database            │
// │  5. run_migrations()       ← buat/update tabel           │
// │  6. build_app()            ← rakit router + middleware   │
// │  7. axum::serve()          ← jalankan server             │
// └─────────────────────────────────────────────────────────┘
// =============================================================================

use std::sync::Arc;

use learn_rust_axum::common::config::Config;
use learn_rust_axum::common::db;
use learn_rust_axum::{build_app, AppState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // ── 1. Load .env ─────────────────────────────────────────────────────
    // dotenvy membaca file .env dan memasukkan isinya ke environment variables.
    // .ok() mengabaikan error jika .env tidak ada (misal di production).
    dotenvy::dotenv().ok();

    // ── 2. Inisialisasi Logging ──────────────────────────────────────────
    // Baca RUST_LOG dari env var untuk menentukan log level.
    // Contoh: RUST_LOG=debug atau RUST_LOG=learn_rust_axum=debug,tower_http=info
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "learn_rust_axum=info,tower_http=info".parse().unwrap()),
        )
        .init();

    tracing::info!("🚀 Starting learn-rust-axum server...");

    // ── 3. Load Konfigurasi ──────────────────────────────────────────────
    let config = Config::from_env();
    tracing::info!("📋 Config loaded (port: {})", config.port);

    // ── 4. Koneksi Database ──────────────────────────────────────────────
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("❌ Failed to create database pool");

    // ── 5. Jalankan Migrations ───────────────────────────────────────────
    db::run_migrations(&pool).await;

    // ── 6. Build Application ─────────────────────────────────────────────
    // Arc (Atomic Reference Count) memungkinkan AppState di-share
    // ke semua handler tanpa cloning seluruh isinya.
    let state = Arc::new(AppState { db: pool, config: config.clone() });
    let app = build_app(state);

    // ── 7. Jalankan Server ───────────────────────────────────────────────
    let addr = config.server_addr();
    let listener = TcpListener::bind(&addr)
        .await
        .expect("❌ Failed to bind to address");

    tracing::info!("✅ Server listening on http://{}", addr);
    tracing::info!("📖 Swagger UI: http://{}/swagger-ui/", addr);
    tracing::info!("📋 Endpoints:");
    tracing::info!("   GET    /health             → Health check");
    tracing::info!("   POST   /api/auth/register  → Register");
    tracing::info!("   POST   /api/auth/login     → Login");
    tracing::info!("   GET    /api/auth/me         → Profile");
    tracing::info!("   POST   /api/tasks           → Create task");
    tracing::info!("   GET    /api/tasks           → List tasks");
    tracing::info!("   GET    /api/tasks/{{id}}      → Get task");
    tracing::info!("   PUT    /api/tasks/{{id}}      → Update task");
    tracing::info!("   DELETE /api/tasks/{{id}}      → Delete task");

    // Jalankan server dengan graceful shutdown
    // Ctrl+C akan menghentikan server dengan bersih (menunggu request selesai)
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("❌ Server failed");
}

/// Menangkap sinyal shutdown (Ctrl+C).
///
/// Fungsi ini akan "menunggu" sampai user menekan Ctrl+C,
/// kemudian return — yang membuat server berhenti menerima request baru
/// dan menunggu request yang sedang berjalan selesai.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    tracing::info!("🛑 Shutdown signal received, gracefully stopping...");
}
