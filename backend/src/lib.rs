// =============================================================================
// lib.rs — Application Core
// =============================================================================
//
// File ini adalah "pusat" dari aplikasi. Di Rust, `lib.rs` berperan sebagai
// library crate yang bisa diakses dari `main.rs` dan juga dari `tests/`.
//
// ┌─────────────────────────────────────────────────────────┐
// │                      ARSITEKTUR                          │
// │                                                          │
// │  main.rs ──→ lib.rs (AppState + build_app)              │
// │                 ├── common/                              │
// │                 │    ├── config.rs  (konfigurasi)        │
// │                 │    ├── db.rs      (database pool)      │
// │                 │    └── errors.rs  (error handling)     │
// │                 ├── models/      (data & query DB)       │
// │                 ├── handlers/    (logika endpoint)       │
// │                 ├── middleware/  (auth, logging)         │
// │                 ├── routes/      (definisi URL)          │
// │                 └── utils/       (helper functions)      │
// └─────────────────────────────────────────────────────────┘
//
// 📝 CARA MENAMBAH MODULE BARU:
//   1. Buat file/folder baru di src/ (misal: src/services/mod.rs)
//   2. Tambahkan `pub mod services;` di bawah ini
//   3. Sekarang bisa diakses dari mana saja sebagai `crate::services::...`
// =============================================================================

// ── Module Declarations ──────────────────────────────────────────────────────
// `pub mod` artinya module ini bisa diakses dari luar crate (dari main.rs, tests)
pub mod common;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod utils;

// ── Imports ──────────────────────────────────────────────────────────────────
use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::common::config::Config;

// =============================================================================
// AppState — Shared Application State
// =============================================================================
//
// Struct ini menyimpan data yang dibutuhkan oleh semua handler.
// Di Axum, state di-share menggunakan `Arc<AppState>` (Atomic Reference Count)
// agar bisa diakses dari multiple threads secara aman.
//
// 💡 TIP: Jika kamu perlu menambah state baru (misal: Redis client, S3 client),
//         tambahkan field baru di sini.
//
// 📝 CARA MENAMBAH STATE BARU:
//   1. Tambah field di struct ini:  `pub redis: RedisPool,`
//   2. Inisialisasi di main.rs saat membuat AppState
//   3. Akses di handler: `let redis = &state.redis;`
// =============================================================================
#[derive(Clone)]
pub struct AppState {
    /// Connection pool ke PostgreSQL
    pub db: PgPool,
    /// Konfigurasi aplikasi (JWT secret, dll)
    pub config: Config,
}

// =============================================================================
// OpenAPI Documentation (Swagger)
// =============================================================================
//
// Struct ini mendefinisikan semua endpoint dan schema yang akan muncul
// di Swagger UI (http://localhost:3000/swagger-ui/).
//
// 📝 CARA MENAMBAH ENDPOINT KE SWAGGER:
//   1. Tambahkan #[utoipa::path(...)] di handler function
//   2. Daftarkan path handler di `paths(...)` di bawah
//   3. Daftarkan request/response struct di `schemas(...)` di bawah
// =============================================================================
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Task Management API",
        version = "1.0.0",
        description = "REST API untuk mengelola tasks — Proyek belajar Rust + Axum + PostgreSQL"
    ),
    paths(
        handlers::health::health_check,
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::me,
        handlers::task::create_task,
        handlers::task::list_tasks,
        handlers::task::get_task,
        handlers::task::update_task,
        handlers::task::delete_task,
    ),
    components(schemas(
        handlers::health::HealthResponse,
        models::user::CreateUserRequest,
        models::user::LoginRequest,
        models::user::UserResponse,
        handlers::auth::AuthResponse,
        models::task::CreateTaskRequest,
        models::task::UpdateTaskRequest,
        models::task::TaskResponse,
        models::task::TaskListResponse,
        common::errors::ErrorResponse,
        common::errors::ErrorBody,
    )),
    tags(
        (name = "health", description = "Health Check"),
        (name = "auth", description = "Authentication — Register, Login, Profile"),
        (name = "tasks", description = "Task Management — CRUD Operations"),
    )
)]
struct ApiDoc;

// =============================================================================
// build_app() — Membangun Router Aplikasi
// =============================================================================
//
// Fungsi ini merakit seluruh aplikasi: routes + middleware + state.
// Dipanggil dari main.rs dan juga dari integration tests.
//
// 📝 CARA MENAMBAH MIDDLEWARE BARU:
//   1. Buat middleware di src/middleware/
//   2. Tambahkan `.layer(...)` di fungsi ini
//   3. Urutan `.layer()` penting! Layer terakhir dieksekusi PERTAMA.
//      (Bayangkan seperti bawang — layer luar dieksekusi duluan)
// =============================================================================
pub fn build_app(state: Arc<AppState>) -> Router {
    // ── CORS Layer ───────────────────────────────────────────────────────
    // CORS (Cross-Origin Resource Sharing) mengizinkan frontend dari
    // domain berbeda mengakses API ini.
    // ⚠️  `Any` hanya untuk development. Di production, batasi origin-nya!
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // ── Timeout Layer ────────────────────────────────────────────────────
    // Request yang lebih dari 30 detik akan otomatis di-cancel.
    #[allow(deprecated)]
    let timeout = TimeoutLayer::new(std::time::Duration::from_secs(30));

    // ── Trace Layer ──────────────────────────────────────────────────────
    // Log setiap request & response (method, path, status, duration).
    let trace = TraceLayer::new_for_http();

    // ── Build Router ─────────────────────────────────────────────────────
    Router::new()
        // Gabungkan semua routes dari module routes/
        .merge(routes::create_routes())
        // Swagger UI — akses di http://localhost:3000/swagger-ui/
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        // Tambahkan middleware layers
        // ⚠️  Urutan: yang TERAKHIR ditambahkan akan dieksekusi PERTAMA
        .layer(cors)
        .layer(timeout)
        .layer(trace)
        // Set shared state — harus dipanggil TERAKHIR
        .with_state(state)
}
