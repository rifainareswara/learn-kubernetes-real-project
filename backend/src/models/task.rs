// =============================================================================
// models/task.rs — Task Data Model & Database Queries
// =============================================================================
//
// FASE 4: CRUD API
//
// File ini berisi model Task lengkap dengan:
//   - Database model (mapping ke tabel tasks)
//   - Request DTOs (Create, Update)
//   - Response DTOs (single task, paginated list)
//   - Database query functions (CRUD + pagination)
//
// Konsep penting yang dipelajari:
//   - Option<T>: Field opsional (boleh null di DB, boleh tidak dikirim client)
//   - Pagination: Limit + Offset untuk membatasi jumlah data yang di-fetch
//   - COALESCE: SQL function untuk "update hanya field yang dikirim"
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

// =============================================================================
// Task Model — Mapping ke tabel `tasks`
// =============================================================================

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Task {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    /// Option<String> = bisa null di database
    pub description: Option<String>,
    /// Status: "pending", "in_progress", "completed"
    pub status: String,
    /// Priority: "low", "medium", "high"
    pub priority: String,
    /// Deadline (opsional)
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Request DTOs
// =============================================================================

/// Data untuk membuat task baru (POST /api/tasks).
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTaskRequest {
    /// Judul task (wajib, 1-255 karakter)
    #[validate(length(min = 1, max = 255, message = "Title harus 1-255 karakter"))]
    pub title: String,

    /// Deskripsi detail (opsional)
    pub description: Option<String>,

    /// Priority: "low", "medium", "high" (default: "medium")
    pub priority: Option<String>,

    /// Deadline (opsional, format: ISO 8601)
    pub due_date: Option<DateTime<Utc>>,
}

/// Data untuk update task (PUT /api/tasks/:id).
///
/// Semua field opsional — hanya field yang dikirim yang akan di-update.
/// Ini disebut "partial update" atau "PATCH-style update".
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    /// Status baru: "pending", "in_progress", "completed"
    pub status: Option<String>,
    /// Priority baru: "low", "medium", "high"
    pub priority: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
}

/// Query parameters untuk list tasks (GET /api/tasks?status=pending&page=1).
///
/// `#[derive(IntoParams)]` dari utoipa mendaftarkan query params ke Swagger.
#[derive(Debug, Deserialize, IntoParams)]
pub struct TaskListQuery {
    /// Filter berdasarkan status
    pub status: Option<String>,
    /// Filter berdasarkan priority
    pub priority: Option<String>,
    /// Nomor halaman (default: 1)
    pub page: Option<i64>,
    /// Jumlah item per halaman (default: 10, max: 100)
    pub per_page: Option<i64>,
}

// =============================================================================
// Response DTOs
// =============================================================================

/// Response untuk single task.
#[derive(Debug, Serialize, ToSchema)]
pub struct TaskResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Konversi Task → TaskResponse (menyembunyikan user_id dari response).
impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            status: task.status,
            priority: task.priority,
            due_date: task.due_date,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }
}

/// Response untuk list tasks dengan pagination info.
#[derive(Debug, Serialize, ToSchema)]
pub struct TaskListResponse {
    /// Daftar task di halaman ini
    pub data: Vec<TaskResponse>,
    /// Total semua task (termasuk yang di halaman lain)
    pub total: i64,
    /// Halaman saat ini
    pub page: i64,
    /// Jumlah item per halaman
    pub per_page: i64,
    /// Total halaman
    pub total_pages: i64,
}

// =============================================================================
// Database Queries
// =============================================================================
//
// 📝 CARA MENAMBAH QUERY DATABASE BARU:
//
//   impl Task {
//       pub async fn find_overdue(pool: &PgPool, user_id: Uuid) -> Result<Vec<Self>, sqlx::Error> {
//           sqlx::query_as::<_, Self>(
//               "SELECT * FROM tasks WHERE user_id = $1 AND due_date < NOW() AND status != 'completed'"
//           )
//           .bind(user_id)
//           .fetch_all(pool)
//           .await
//       }
//   }
// =============================================================================

impl Task {
    /// Membuat task baru untuk user tertentu.
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        req: &CreateTaskRequest,
    ) -> Result<Self, sqlx::Error> {
        // unwrap_or: jika priority tidak dikirim, pakai "medium"
        let priority = req.priority.as_deref().unwrap_or("medium");

        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO tasks (user_id, title, description, priority, due_date)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(priority)
        .bind(req.due_date)
        .fetch_one(pool)
        .await
    }

    /// Ambil satu task berdasarkan ID.
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Ambil semua task milik user dengan pagination dan filter.
    ///
    /// # Pagination
    /// - `page`: nomor halaman (mulai dari 1)
    /// - `per_page`: jumlah item per halaman
    /// - `LIMIT`: batasi jumlah row yang diambil
    /// - `OFFSET`: skip row sebanyak (page - 1) * per_page
    ///
    /// # Returns
    /// Tuple `(Vec<Task>, total_count)` — tasks + total untuk pagination info.
    pub async fn find_all_by_user(
        pool: &PgPool,
        user_id: Uuid,
        query: &TaskListQuery,
    ) -> Result<(Vec<Self>, i64), sqlx::Error> {
        // Hitung pagination
        let page = query.page.unwrap_or(1).max(1); // minimal halaman 1
        let per_page = query.per_page.unwrap_or(10).clamp(1, 100); // 1-100 item
        let offset = (page - 1) * per_page;

        // Fetch tasks berdasarkan filter
        let tasks = match (&query.status, &query.priority) {
            // Filter status DAN priority
            (Some(status), Some(priority)) => {
                sqlx::query_as::<_, Self>(
                    r#"SELECT * FROM tasks
                       WHERE user_id = $1 AND status = $2 AND priority = $3
                       ORDER BY created_at DESC
                       LIMIT $4 OFFSET $5"#,
                )
                .bind(user_id)
                .bind(status)
                .bind(priority)
                .bind(per_page)
                .bind(offset)
                .fetch_all(pool)
                .await?
            }
            // Filter status saja
            (Some(status), None) => {
                sqlx::query_as::<_, Self>(
                    r#"SELECT * FROM tasks
                       WHERE user_id = $1 AND status = $2
                       ORDER BY created_at DESC
                       LIMIT $3 OFFSET $4"#,
                )
                .bind(user_id)
                .bind(status)
                .bind(per_page)
                .bind(offset)
                .fetch_all(pool)
                .await?
            }
            // Filter priority saja
            (None, Some(priority)) => {
                sqlx::query_as::<_, Self>(
                    r#"SELECT * FROM tasks
                       WHERE user_id = $1 AND priority = $2
                       ORDER BY created_at DESC
                       LIMIT $3 OFFSET $4"#,
                )
                .bind(user_id)
                .bind(priority)
                .bind(per_page)
                .bind(offset)
                .fetch_all(pool)
                .await?
            }
            // Tanpa filter
            (None, None) => {
                sqlx::query_as::<_, Self>(
                    r#"SELECT * FROM tasks
                       WHERE user_id = $1
                       ORDER BY created_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(user_id)
                .bind(per_page)
                .bind(offset)
                .fetch_all(pool)
                .await?
            }
        };

        // Hitung total untuk pagination info
        let total: (i64,) = match (&query.status, &query.priority) {
            (Some(status), Some(priority)) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND status = $2 AND priority = $3",
                )
                .bind(user_id)
                .bind(status)
                .bind(priority)
                .fetch_one(pool)
                .await?
            }
            (Some(status), None) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND status = $2",
                )
                .bind(user_id)
                .bind(status)
                .fetch_one(pool)
                .await?
            }
            (None, Some(priority)) => {
                sqlx::query_as(
                    "SELECT COUNT(*) FROM tasks WHERE user_id = $1 AND priority = $2",
                )
                .bind(user_id)
                .bind(priority)
                .fetch_one(pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as("SELECT COUNT(*) FROM tasks WHERE user_id = $1")
                    .bind(user_id)
                    .fetch_one(pool)
                    .await?
            }
        };

        Ok((tasks, total.0))
    }

    /// Update task (partial update — hanya field yang dikirim yang berubah).
    ///
    /// `COALESCE($1, title)` artinya: gunakan $1 jika tidak NULL,
    /// kalau NULL gunakan nilai yang sudah ada (title lama).
    ///
    /// `AND user_id = $7` memastikan user hanya bisa update task miliknya sendiri.
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        user_id: Uuid,
        req: &UpdateTaskRequest,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            UPDATE tasks SET
                title = COALESCE($1, title),
                description = COALESCE($2, description),
                status = COALESCE($3, status),
                priority = COALESCE($4, priority),
                due_date = COALESCE($5, due_date),
                updated_at = NOW()
            WHERE id = $6 AND user_id = $7
            RETURNING *
            "#,
        )
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.status)
        .bind(&req.priority)
        .bind(req.due_date)
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Hapus task berdasarkan ID.
    ///
    /// `AND user_id = $2` memastikan user hanya bisa hapus task miliknya.
    /// `rows_affected()` mengembalikan jumlah row yang terhapus (0 atau 1).
    pub async fn delete(pool: &PgPool, id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
