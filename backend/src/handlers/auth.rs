// =============================================================================
// handlers/auth.rs — Authentication Handlers
// =============================================================================
//
// FASE 5: Authentication
//
// File ini berisi 3 handler untuk authentication:
//   POST /api/auth/register  → Daftar user baru
//   POST /api/auth/login     → Login dan mendapat JWT token
//   GET  /api/auth/me        → Lihat profil user yang sedang login
//
// Flow Authentication:
//   1. User register → password di-hash → simpan ke DB
//   2. User login → verifikasi password → buat JWT token → kirim ke client
//   3. Client kirim JWT di header Authorization untuk akses endpoint protected
//
// 📝 CARA MEMBUAT HANDLER BARU:
//
//   // 1. Definisikan response struct (jika perlu)
//   #[derive(Serialize, ToSchema)]
//   pub struct MyResponse { pub data: String }
//
//   // 2. Tulis handler function
//   #[utoipa::path(post, path = "/api/my-endpoint", ...)]
//   pub async fn my_handler(
//       State(state): State<Arc<AppState>>,    // ← akses database & config
//       Json(body): Json<MyRequest>,            // ← parse body JSON
//   ) -> Result<Json<MyResponse>, AppError> {   // ← return JSON atau error
//       // 3. Validasi input
//       body.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;
//
//       // 4. Logika bisnis (query DB, proses data, dll)
//       let result = do_something(&state.db).await?;
//
//       // 5. Return response
//       Ok(Json(MyResponse { data: result }))
//   }
//
//   // 6. Tambahkan route di src/routes/
//   // 7. Daftarkan di swagger (lib.rs → ApiDoc)
// =============================================================================

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use validator::Validate;

use crate::common::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::user::{CreateUserRequest, LoginRequest, User, UserResponse};
use crate::utils::jwt;
use crate::utils::password;
use crate::AppState;

// =============================================================================
// Response DTO
// =============================================================================

/// Response setelah register/login berhasil.
///
/// Berisi JWT token untuk dipakai di request selanjutnya.
#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    /// JWT token (kirim di header: Authorization: Bearer <token>)
    pub token: String,
    /// Data user yang login
    pub user: UserResponse,
}

// =============================================================================
// POST /api/auth/register — Registrasi User Baru
// =============================================================================

/// Registrasi user baru.
///
/// Menerima username, email, dan password. Password akan di-hash
/// menggunakan Argon2 sebelum disimpan ke database.
///
/// # Request Body
/// ```json
/// {
///   "username": "john_doe",
///   "email": "john@example.com",
///   "password": "min8chars"
/// }
/// ```
///
/// # Response (201 Created)
/// ```json
/// {
///   "token": "eyJhbGci...",
///   "user": { "id": "...", "username": "john_doe", ... }
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User berhasil didaftarkan", body = AuthResponse),
        (status = 409, description = "Email atau username sudah terdaftar"),
        (status = 422, description = "Validasi input gagal"),
    )
)]
pub async fn register(
    // State: shared application state (DB pool, config)
    State(state): State<Arc<AppState>>,
    // Json: parse request body sebagai JSON → CreateUserRequest
    Json(body): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    // 1. Validasi input (email format, password length, dll)
    //    `.validate()` dari crate `validator` mengecek semua #[validate(...)]
    //    `.map_err(...)` mengkonversi ValidationErrors → AppError
    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // 2. Cek apakah email sudah terdaftar
    if User::find_by_email(&state.db, &body.email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Email sudah terdaftar".to_string()));
    }

    // 3. Cek apakah username sudah dipakai
    if User::username_exists(&state.db, &body.username).await? {
        return Err(AppError::Conflict("Username sudah dipakai".to_string()));
    }

    // 4. Hash password (JANGAN simpan plain text!)
    let password_hash = password::hash_password(&body.password)
        .map_err(|e| AppError::InternalServerError(format!("Hash error: {}", e)))?;

    // 5. Simpan user ke database
    let user = User::create(&state.db, &body.username, &body.email, &password_hash).await?;

    // 6. Buat JWT token
    let token = jwt::create_token(user.id, &state.config.jwt_secret)
        .map_err(|e| AppError::InternalServerError(format!("Token error: {}", e)))?;

    // 7. Return response dengan status 201 Created
    //    Tuple (StatusCode, Json<T>) otomatis di-convert ke Response
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: UserResponse::from(user), // User → UserResponse (tanpa password)
        }),
    ))
}

// =============================================================================
// POST /api/auth/login — Login
// =============================================================================

/// Login user dan mendapatkan JWT token.
///
/// # Request Body
/// ```json
/// {
///   "email": "john@example.com",
///   "password": "min8chars"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login berhasil", body = AuthResponse),
        (status = 401, description = "Email atau password salah"),
    )
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // 1. Cari user berdasarkan email
    //    .ok_or_else() mengkonversi None → Error
    let user = User::find_by_email(&state.db, &body.email)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Email atau password salah".to_string()))?;

    // 2. Verifikasi password
    let is_valid = password::verify_password(&body.password, &user.password_hash)
        .map_err(|e| AppError::InternalServerError(format!("Verify error: {}", e)))?;

    if !is_valid {
        // ⚠️  Pesan error SAMA dengan "email tidak ditemukan"
        //     untuk mencegah user enumeration attack
        return Err(AppError::Unauthorized(
            "Email atau password salah".to_string(),
        ));
    }

    // 3. Buat JWT token
    let token = jwt::create_token(user.id, &state.config.jwt_secret)
        .map_err(|e| AppError::InternalServerError(format!("Token error: {}", e)))?;

    // 4. Return token + user data
    Ok(Json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}

// =============================================================================
// GET /api/auth/me — Get Current User Profile
// =============================================================================

/// Mendapatkan profil user yang sedang login.
///
/// Endpoint ini PROTECTED — membutuhkan JWT token di header Authorization.
/// Parameter `auth: AuthUser` otomatis memvalidasi token.
///
/// # Headers
/// ```
/// Authorization: Bearer eyJhbGci...
/// ```
#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "auth",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Profil user", body = UserResponse),
        (status = 401, description = "Token tidak valid / belum login"),
    )
)]
pub async fn me(
    State(state): State<Arc<AppState>>,
    // AuthUser: custom extractor yang memvalidasi JWT token
    // Jika token invalid → otomatis return 401 (tidak sampai ke handler)
    auth: AuthUser,
) -> Result<Json<UserResponse>, AppError> {
    // Ambil user dari DB berdasarkan user_id di token
    let user = User::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User tidak ditemukan".to_string()))?;

    Ok(Json(UserResponse::from(user)))
}
