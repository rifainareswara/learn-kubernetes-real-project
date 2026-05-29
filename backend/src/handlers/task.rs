// =============================================================================
// handlers/task.rs — Task CRUD Handlers
// =============================================================================
//
// FASE 4: CRUD API
//
// File ini berisi 5 handler untuk operasi CRUD (Create, Read, Update, Delete):
//
//   POST   /api/tasks      → Create task baru
//   GET    /api/tasks      → List semua task (+ pagination & filter)
//   GET    /api/tasks/:id  → Get detail satu task
//   PUT    /api/tasks/:id  → Update task
//   DELETE /api/tasks/:id  → Hapus task
//
// Semua endpoint PROTECTED — membutuhkan JWT token.
// User hanya bisa mengakses task miliknya sendiri (authorization).
//
// ┌────────────────────────────────────────────────────────┐
// │              HTTP METHOD → CRUD MAPPING                 │
// │                                                         │
// │  POST   = Create  (buat resource baru)                  │
// │  GET    = Read    (baca/ambil resource)                  │
// │  PUT    = Update  (ubah resource yang sudah ada)         │
// │  DELETE = Delete  (hapus resource)                       │
// │                                                         │
// │  Status Code Conventions:                                │
// │  200 OK         = Sukses (GET, PUT)                      │
// │  201 Created    = Resource baru berhasil dibuat (POST)   │
// │  204 No Content = Berhasil tanpa response body (DELETE)  │
// │  404 Not Found  = Resource tidak ditemukan                │
// │  403 Forbidden  = Tidak punya akses ke resource ini      │
// └────────────────────────────────────────────────────────┘
// =============================================================================

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;
use validator::Validate;

use crate::common::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::task::{
    CreateTaskRequest, Task, TaskListQuery, TaskListResponse, TaskResponse, UpdateTaskRequest,
};
use crate::AppState;

// =============================================================================
// POST /api/tasks — Create Task
// =============================================================================

/// Membuat task baru.
///
/// Task otomatis di-assign ke user yang sedang login.
///
/// # Request Body
/// ```json
/// {
///   "title": "Belajar Rust",
///   "description": "Baca buku The Rust Programming Language",
///   "priority": "high",
///   "due_date": "2026-06-01T00:00:00Z"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/tasks",
    tag = "tasks",
    security(("bearer_auth" = [])),
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task berhasil dibuat", body = TaskResponse),
        (status = 422, description = "Validasi gagal"),
    )
)]
pub async fn create_task(
    State(state): State<Arc<AppState>>,
    auth: AuthUser, // ← Protected endpoint
    Json(body): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), AppError> {
    // Validasi input
    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Validasi priority (jika dikirim)
    if let Some(ref priority) = body.priority {
        if !["low", "medium", "high"].contains(&priority.as_str()) {
            return Err(AppError::BadRequest(
                "Priority harus: low, medium, atau high".to_string(),
            ));
        }
    }

    // Buat task — user_id diambil dari JWT token
    let task = Task::create(&state.db, auth.user_id, &body).await?;

    Ok((StatusCode::CREATED, Json(TaskResponse::from(task))))
}

// =============================================================================
// GET /api/tasks — List Tasks (with Pagination)
// =============================================================================

/// Mendapatkan daftar task milik user yang login.
///
/// Mendukung pagination dan filter.
///
/// # Query Parameters
/// - `status`: Filter berdasarkan status (pending/in_progress/completed)
/// - `priority`: Filter berdasarkan priority (low/medium/high)
/// - `page`: Nomor halaman (default: 1)
/// - `per_page`: Jumlah item per halaman (default: 10, max: 100)
///
/// # Contoh
/// ```
/// GET /api/tasks?status=pending&page=1&per_page=5
/// ```
#[utoipa::path(
    get,
    path = "/api/tasks",
    tag = "tasks",
    security(("bearer_auth" = [])),
    params(TaskListQuery),
    responses(
        (status = 200, description = "Daftar task", body = TaskListResponse),
    )
)]
pub async fn list_tasks(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    // Query: extractor untuk query string parameters (?key=value)
    Query(query): Query<TaskListQuery>,
) -> Result<Json<TaskListResponse>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(10).clamp(1, 100);

    // Fetch tasks + total count untuk pagination
    let (tasks, total) = Task::find_all_by_user(&state.db, auth.user_id, &query).await?;

    // Hitung total halaman (pembulatan ke atas)
    let total_pages = (total as f64 / per_page as f64).ceil() as i64;

    Ok(Json(TaskListResponse {
        data: tasks.into_iter().map(TaskResponse::from).collect(),
        total,
        page,
        per_page,
        total_pages,
    }))
}

// =============================================================================
// GET /api/tasks/:id — Get Single Task
// =============================================================================

/// Mendapatkan detail satu task berdasarkan ID.
///
/// # Path Parameters
/// - `id`: UUID task
///
/// # Authorization
/// User hanya bisa melihat task miliknya sendiri.
#[utoipa::path(
    get,
    path = "/api/tasks/{id}",
    tag = "tasks",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Task ID (UUID)")
    ),
    responses(
        (status = 200, description = "Detail task", body = TaskResponse),
        (status = 404, description = "Task tidak ditemukan"),
    )
)]
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    // Path: extractor untuk path parameters (/tasks/{id} → id)
    Path(id): Path<Uuid>,
) -> Result<Json<TaskResponse>, AppError> {
    // Cari task di database
    let task = Task::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Task tidak ditemukan".to_string()))?;

    // Authorization: pastikan task milik user yang login
    if task.user_id != auth.user_id {
        return Err(AppError::Forbidden(
            "Kamu tidak punya akses ke task ini".to_string(),
        ));
    }

    Ok(Json(TaskResponse::from(task)))
}

// =============================================================================
// PUT /api/tasks/:id — Update Task
// =============================================================================

/// Mengupdate task (partial update).
///
/// Hanya field yang dikirim yang akan diubah. Field yang tidak dikirim
/// tetap menggunakan nilai lama.
///
/// # Request Body
/// ```json
/// {
///   "status": "completed",
///   "title": "Judul baru"
/// }
/// ```
#[utoipa::path(
    put,
    path = "/api/tasks/{id}",
    tag = "tasks",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Task ID (UUID)")
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Task berhasil diupdate", body = TaskResponse),
        (status = 404, description = "Task tidak ditemukan"),
    )
)]
pub async fn update_task(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    // Validasi status (jika dikirim)
    if let Some(ref status) = body.status {
        if !["pending", "in_progress", "completed"].contains(&status.as_str()) {
            return Err(AppError::BadRequest(
                "Status harus: pending, in_progress, atau completed".to_string(),
            ));
        }
    }

    // Validasi priority (jika dikirim)
    if let Some(ref priority) = body.priority {
        if !["low", "medium", "high"].contains(&priority.as_str()) {
            return Err(AppError::BadRequest(
                "Priority harus: low, medium, atau high".to_string(),
            ));
        }
    }

    // Update task — SQL query sudah include `AND user_id = $7`
    // jadi jika task bukan milik user, hasilnya None
    let task = Task::update(&state.db, id, auth.user_id, &body)
        .await?
        .ok_or_else(|| AppError::NotFound("Task tidak ditemukan".to_string()))?;

    Ok(Json(TaskResponse::from(task)))
}

// =============================================================================
// DELETE /api/tasks/:id — Delete Task
// =============================================================================

/// Menghapus task.
///
/// # Response
/// - 204 No Content: Task berhasil dihapus
/// - 404 Not Found: Task tidak ditemukan / bukan milik user
#[utoipa::path(
    delete,
    path = "/api/tasks/{id}",
    tag = "tasks",
    security(("bearer_auth" = [])),
    params(
        ("id" = Uuid, Path, description = "Task ID (UUID)")
    ),
    responses(
        (status = 204, description = "Task berhasil dihapus"),
        (status = 404, description = "Task tidak ditemukan"),
    )
)]
pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Delete — SQL query include `AND user_id = $2` untuk authorization
    let deleted = Task::delete(&state.db, id, auth.user_id).await?;

    if !deleted {
        return Err(AppError::NotFound("Task tidak ditemukan".to_string()));
    }

    // 204 No Content — tidak ada body response
    Ok(StatusCode::NO_CONTENT)
}
