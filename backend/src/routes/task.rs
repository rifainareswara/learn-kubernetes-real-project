// =============================================================================
// routes/task.rs — Task Management Routes
// =============================================================================
//
// Route definitions untuk task CRUD endpoints.
//
// 💡 TIP: Satu `.route()` bisa menangani MULTIPLE HTTP methods:
//   .route("/", get(list).post(create))
//   artinya:
//     GET  /  → list handler
//     POST /  → create handler
//
// Path parameter di Axum 0.8 menggunakan syntax `{param}`:
//   .route("/{id}", ...) → handler menerima `Path(id): Path<Uuid>`
// =============================================================================

use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use crate::handlers;
use crate::AppState;

/// Route group untuk task management.
///
/// Di-nest di bawah prefix `/api/tasks` (lihat routes/mod.rs).
///
/// Hasil akhir:
///   POST   /api/tasks       → Create task
///   GET    /api/tasks       → List tasks (+ pagination)
///   GET    /api/tasks/{id}  → Get single task
///   PUT    /api/tasks/{id}  → Update task
///   DELETE /api/tasks/{id}  → Delete task
pub fn task_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Collection routes (tanpa :id)
        .route(
            "/",
            get(handlers::task::list_tasks).post(handlers::task::create_task),
        )
        // Item routes (dengan {id})
        .route(
            "/{id}",
            get(handlers::task::get_task)
                .put(handlers::task::update_task)
                .delete(handlers::task::delete_task),
        )
}
