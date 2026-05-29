// =============================================================================
// errors.rs — Centralized Error Handling
// =============================================================================
//
// FASE 6: Error Handling
//
// Modul ini mendefinisikan SATU tipe error (`AppError`) yang dipakai
// di seluruh aplikasi. Semua error dikonversi ke JSON response yang konsisten.
//
// Kenapa centralized error handling?
//   - Konsisten: semua error punya format JSON yang sama
//   - Mudah di-maintain: satu tempat untuk mengubah format error
//   - Type-safe: compiler memastikan semua error di-handle
//
// Format JSON error response:
//   {
//     "error": {
//       "code": 404,
//       "message": "Task not found"
//     }
//   }
//
// 📝 CARA MENAMBAH ERROR BARU:
//   1. Tambah variant baru di enum AppError (contoh: `RateLimited(String)`)
//   2. Tambahkan mapping di impl IntoResponse (status code + message)
//   3. Gunakan di handler: `return Err(AppError::RateLimited("...".into()))`
// =============================================================================

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

// =============================================================================
// AppError — Custom Error Type
// =============================================================================
//
// `#[derive(thiserror::Error)]` otomatis mengimplementasikan:
//   - std::fmt::Display (pesan error)
//   - std::error::Error trait
//
// `#[error("...")]` mendefinisikan pesan yang ditampilkan oleh Display.
// `#[from]` otomatis mengkonversi dari tipe error lain (implicit conversion).
// =============================================================================
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// 400 Bad Request — Input tidak valid secara umum
    #[error("{0}")]
    BadRequest(String),

    /// 401 Unauthorized — Belum login / token invalid
    #[error("{0}")]
    Unauthorized(String),

    /// 403 Forbidden — Sudah login tapi tidak punya akses
    #[error("{0}")]
    Forbidden(String),

    /// 404 Not Found — Resource tidak ditemukan
    #[error("{0}")]
    NotFound(String),

    /// 409 Conflict — Resource sudah ada (misal: email sudah terdaftar)
    #[error("{0}")]
    Conflict(String),

    /// 422 Unprocessable Entity — Validasi input gagal
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// 500 Internal Server Error — Error tak terduga
    #[error("{0}")]
    InternalServerError(String),

    /// Error dari database (SQLx) — otomatis dikonversi berkat `#[from]`
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

// =============================================================================
// Error Response Structs
// =============================================================================

/// Format JSON untuk semua error response.
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

/// Body dari error response.
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    /// HTTP status code (contoh: 404)
    pub code: u16,
    /// Pesan error yang human-readable
    pub message: String,
}

// =============================================================================
// IntoResponse — Konversi AppError ke HTTP Response
// =============================================================================
//
// Trait `IntoResponse` dari Axum memungkinkan return type handler berupa
// `Result<..., AppError>`. Axum otomatis memanggil `.into_response()` jika error.
//
// 💡 TIP: Di Rust, `impl Trait for Struct` artinya kita "mengajarkan"
//         struct kita cara melakukan sesuatu yang didefinisikan oleh trait.
//         Di sini, kita mengajarkan AppError cara mengubah diri menjadi HTTP response.
// =============================================================================
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Pattern matching — cara Rust menangani enum variants
        let (status, message) = match &self {
            // Error dari kode kita sendiri — langsung pakai pesan yang diberikan
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::InternalServerError(msg) => {
                // Log error internal (jangan tampilkan detail ke user!)
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            // Error dari SQLx — konversi ke HTTP status yang sesuai
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {}", e);
                match e {
                    // Row tidak ditemukan → 404
                    sqlx::Error::RowNotFound => {
                        (StatusCode::NOT_FOUND, "Resource not found".to_string())
                    }
                    // Error database spesifik (constraint violation, dll)
                    sqlx::Error::Database(db_err) => {
                        // PostgreSQL error code 23505 = unique_violation
                        if db_err.code().as_deref() == Some("23505") {
                            (StatusCode::CONFLICT, "Resource already exists".to_string())
                        } else {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Internal server error".to_string(),
                            )
                        }
                    }
                    // Error lainnya → 500
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error".to_string(),
                    ),
                }
            }
        };

        // Buat JSON response body
        let body = ErrorResponse {
            error: ErrorBody {
                code: status.as_u16(),
                message,
            },
        };

        // Return tuple (StatusCode, Json<Body>) — Axum konversi ini ke Response
        (status, axum::Json(body)).into_response()
    }
}
