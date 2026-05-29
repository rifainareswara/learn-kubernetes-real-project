// =============================================================================
// models/user.rs — User Data Model & Database Queries
// =============================================================================
//
// FASE 4: CRUD API + FASE 5: Authentication
//
// File ini berisi:
//   1. Struct `User`              — mapping dari tabel `users` di PostgreSQL
//   2. Struct `CreateUserRequest` — DTO untuk registrasi user baru
//   3. Struct `LoginRequest`      — DTO untuk login
//   4. Struct `UserResponse`      — DTO untuk response ke client
//   5. Database query functions   — associated functions di `impl User`
//
// 📝 CARA MEMBUAT MODEL BARU (step-by-step):
//
//   // 1. Definisikan struct Model (mapping ke tabel DB)
//   #[derive(Debug, sqlx::FromRow, Serialize)]
//   pub struct Category {
//       pub id: Uuid,
//       pub name: String,
//       pub created_at: DateTime<Utc>,
//   }
//
//   // 2. Definisikan struct Request (data dari client)
//   #[derive(Debug, Deserialize, Validate, ToSchema)]
//   pub struct CreateCategoryRequest {
//       #[validate(length(min = 1, max = 100))]
//       pub name: String,
//   }
//
//   // 3. Definisikan struct Response (data ke client)
//   #[derive(Debug, Serialize, ToSchema)]
//   pub struct CategoryResponse {
//       pub id: Uuid,
//       pub name: String,
//   }
//
//   // 4. Implementasikan database queries
//   impl Category {
//       pub async fn create(pool: &PgPool, name: &str) -> Result<Self, sqlx::Error> {
//           sqlx::query_as::<_, Self>("INSERT INTO ... RETURNING *")
//               .bind(name)
//               .fetch_one(pool)
//               .await
//       }
//   }
//
//   // 5. Implementasikan konversi Model → Response
//   impl From<Category> for CategoryResponse {
//       fn from(c: Category) -> Self {
//           Self { id: c.id, name: c.name }
//       }
//   }
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

// =============================================================================
// User Model — Mapping ke tabel `users`
// =============================================================================
//
// `#[derive(sqlx::FromRow)]` memungkinkan SQLx otomatis mengkonversi
// row dari database ke struct ini. Nama field HARUS sama dengan nama kolom.
//
// `#[derive(Serialize)]` memungkinkan konversi struct → JSON.
// `#[serde(skip_serializing)]` menyembunyikan field dari JSON output.
// =============================================================================

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,

    /// ⚠️  JANGAN pernah kirim password hash ke client!
    /// `skip_serializing` memastikan field ini tidak muncul di JSON.
    #[serde(skip_serializing)]
    pub password_hash: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Request DTOs (Data Transfer Objects)
// =============================================================================

/// Data yang dibutuhkan untuk registrasi user baru.
///
/// `#[derive(Validate)]` mengaktifkan validasi input.
/// `#[derive(ToSchema)]` mendaftarkan struct ini ke Swagger.
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    /// Username harus 3-50 karakter
    #[validate(length(min = 3, max = 50, message = "Username harus 3-50 karakter"))]
    pub username: String,

    /// Email harus format valid (contoh: user@example.com)
    #[validate(email(message = "Format email tidak valid"))]
    pub email: String,

    /// Password minimal 8 karakter
    #[validate(length(min = 8, message = "Password minimal 8 karakter"))]
    pub password: String,
}

/// Data yang dibutuhkan untuk login.
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    /// Email yang terdaftar
    pub email: String,
    /// Password plain text
    pub password: String,
}

// =============================================================================
// Response DTO
// =============================================================================

/// Data user yang dikirim ke client (TANPA password hash).
#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

/// Konversi User (model) → UserResponse (response).
///
/// `impl From<A> for B` memungkinkan konversi dengan:
///   let response: UserResponse = user.into();
///   let response = UserResponse::from(user);
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

// =============================================================================
// Database Queries — Associated Functions
// =============================================================================
//
// `impl User { ... }` menambahkan fungsi-fungsi ke struct User.
// Ini disebut "associated functions" atau "methods".
//
// Pattern yang dipakai di sini:
//   - Fungsi menerima `&PgPool` (reference ke connection pool)
//   - Fungsi async karena database I/O
//   - Return `Result<..., sqlx::Error>` karena query bisa gagal
//
// SQLx query functions:
//   .fetch_one(pool)      → Harus return tepat 1 row (error jika 0 atau >1)
//   .fetch_optional(pool) → Return Option<T> (None jika 0 rows)
//   .fetch_all(pool)      → Return Vec<T> (bisa kosong)
//   .execute(pool)        → Untuk INSERT/UPDATE/DELETE tanpa RETURNING
//
// 💡 TIP: Kita pakai `sqlx::query_as::<_, User>(...)` bukan `sqlx::query_as!(...)`
//         karena versi runtime tidak perlu DATABASE_URL saat compile.
// =============================================================================

impl User {
    /// Membuat user baru di database.
    ///
    /// `RETURNING *` di SQL membuat PostgreSQL mengembalikan row yang baru dibuat,
    /// termasuk field yang di-generate (id, created_at, updated_at).
    pub async fn create(
        pool: &PgPool,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as::<_, Self>(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(pool)
        .await
    }

    /// Cari user berdasarkan email.
    ///
    /// Return `Option<User>`:
    ///   - `Some(user)` jika ditemukan
    ///   - `None` jika tidak ada
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    /// Cari user berdasarkan ID (UUID).
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as::<_, Self>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    /// Cek apakah username sudah dipakai.
    pub async fn username_exists(pool: &PgPool, username: &str) -> Result<bool, sqlx::Error> {
        let result: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
                .bind(username)
                .fetch_one(pool)
                .await?;
        Ok(result.0)
    }
}
