// =============================================================================
// routes/auth.rs — Authentication Routes
// =============================================================================
//
// Route definitions untuk authentication endpoints.
//
// 💡 TIP: Di Axum, route dan handler DIPISAHKAN:
//   - routes/ mendefinisikan URL + HTTP method
//   - handlers/ berisi logika bisnis
//
//   Ini membuat kode lebih terstruktur dan mudah dibaca.
//   Kamu bisa melihat semua URL di routes/ tanpa membaca logika handler.
// =============================================================================

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::handlers;
use crate::AppState;

/// Route group untuk authentication.
///
/// Routes ini akan di-nest di bawah prefix `/api/auth` (lihat routes/mod.rs).
///
/// Hasil akhir:
///   POST /api/auth/register → handlers::auth::register
///   POST /api/auth/login    → handlers::auth::login
///   GET  /api/auth/me       → handlers::auth::me
pub fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route("/me", get(handlers::auth::me))
}
