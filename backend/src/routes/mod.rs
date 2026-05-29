// =============================================================================
// routes/mod.rs — Route Registry & Router Builder
// =============================================================================
//
// FASE 2: Routing
//
// File ini menggabungkan semua route group menjadi satu Router.
//
// Struktur URL:
//   /health          → Health check (public)
//   /api/auth/*      → Authentication (register, login, me)
//   /api/tasks/*     → Task CRUD (protected)
//
// Konsep:
//   - `Router::new()` : Membuat router kosong
//   - `.route(path, handler)` : Menambah route
//   - `.nest(prefix, router)` : Menggabungkan sub-router dengan URL prefix
//   - `.merge(router)` : Menggabungkan router tanpa prefix
//
// `.nest("/api/auth", auth_routes)` artinya:
//   Semua route di auth_routes akan diawali "/api/auth"
//   Jadi jika auth_routes punya "/register", URL finalnya "/api/auth/register"
//
// 📝 CARA MENAMBAH ROUTE GROUP BARU:
//   1. Buat file baru (contoh: src/routes/category.rs)
//   2. Daftarkan di sini: `pub mod category;`
//   3. Tambahkan `.nest("/api/categories", category::category_routes())`
//      di fungsi create_routes()
// =============================================================================

pub mod auth;
pub mod task;

use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use crate::handlers;
use crate::AppState;

/// Membangun router utama dengan semua route.
///
/// Return `Router<Arc<AppState>>` — router yang BELUM di-set state-nya.
/// State akan di-set di `build_app()` di lib.rs.
pub fn create_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Route health check (tanpa prefix)
        .route("/health", get(handlers::health::health_check))
        // Gabungkan sub-routes dengan URL prefix
        .nest("/api/auth", auth::auth_routes())
        .nest("/api/tasks", task::task_routes())
}
