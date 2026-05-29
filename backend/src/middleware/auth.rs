// =============================================================================
// middleware/auth.rs — JWT Authentication Extractor
// =============================================================================
//
// FASE 5: Authentication & Authorization
//
// File ini berisi custom extractor `AuthUser` yang:
//   1. Membaca header `Authorization: Bearer <token>`
//   2. Memverifikasi JWT token
//   3. Mengekstrak user_id dari token
//   4. Mengembalikan AuthUser { user_id } yang bisa dipakai di handler
//
// Cara pakai di handler:
//   async fn my_handler(auth: AuthUser) -> ... {
//       // auth.user_id berisi UUID user yang sedang login
//   }
//
// Jika token tidak valid/expired, Axum otomatis return 401 Unauthorized.
//
// ┌─────────────────────────────────────────────────────────────┐
// │                    FLOW AUTHENTICATION                       │
// │                                                              │
// │  Client                         Server                       │
// │    │                               │                          │
// │    │ ─── Request + Token ────────→ │                          │
// │    │     Authorization: Bearer xxx │                          │
// │    │                               │                          │
// │    │                          ┌────┴────┐                     │
// │    │                          │AuthUser │ ← Extractor         │
// │    │                          │extract  │                     │
// │    │                          │token    │                     │
// │    │                          │verify   │                     │
// │    │                          │JWT      │                     │
// │    │                          └────┬────┘                     │
// │    │                               │                          │
// │    │                          Valid? ─── No ──→ 401 Response  │
// │    │                               │                          │
// │    │                              Yes                         │
// │    │                               │                          │
// │    │                          ┌────┴────┐                     │
// │    │                          │ Handler │ ← Terima AuthUser   │
// │    │                          └────┬────┘                     │
// │    │                               │                          │
// │    │ ←── Response ──────────────── │                          │
// └─────────────────────────────────────────────────────────────┘
//
// 📝 CARA MEMBUAT CUSTOM EXTRACTOR BARU:
//
//   // 1. Definisikan struct
//   pub struct ApiKey(pub String);
//
//   // 2. Implement FromRequestParts
//   impl FromRequestParts<Arc<AppState>> for ApiKey {
//       type Rejection = AppError;
//
//       async fn from_request_parts(
//           parts: &mut Parts,
//           _state: &Arc<AppState>,
//       ) -> Result<Self, Self::Rejection> {
//           let key = parts.headers
//               .get("X-API-Key")
//               .and_then(|v| v.to_str().ok())
//               .ok_or(AppError::Unauthorized("Missing API key".into()))?;
//           Ok(ApiKey(key.to_string()))
//       }
//   }
//
//   // 3. Pakai di handler
//   async fn handler(ApiKey(key): ApiKey) -> ... { ... }
// =============================================================================

use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::common::errors::AppError;
use crate::utils::jwt;
use crate::AppState;

/// Extractor yang mengandung user_id dari JWT token.
///
/// Menambahkan `AuthUser` sebagai parameter handler otomatis membuat
/// endpoint tersebut "protected" — hanya bisa diakses oleh user yang login.
///
/// # Contoh
/// ```rust
/// // Endpoint ini MEMBUTUHKAN login
/// async fn protected(auth: AuthUser) -> String {
///     format!("Hello, user {}", auth.user_id)
/// }
///
/// // Endpoint ini TIDAK membutuhkan login
/// async fn public() -> String {
///     "Hello, anonymous".to_string()
/// }
/// ```
pub struct AuthUser {
    pub user_id: Uuid,
}

// =============================================================================
// FromRequestParts — Implementasi Custom Extractor
// =============================================================================
//
// `FromRequestParts` adalah trait dari Axum untuk membuat extractor.
// Axum memanggil `from_request_parts()` secara otomatis sebelum handler.
//
// Perbedaan FromRequestParts vs FromRequest:
//   - FromRequestParts: hanya akses headers, query params, dll (tanpa body)
//   - FromRequest: akses body juga (consume body, jadi hanya bisa 1x)
//
// Generic parameter `Arc<AppState>`:
//   Ini menentukan tipe state yang bisa diakses oleh extractor.
//   Harus sama dengan tipe state yang di-set di Router.
// =============================================================================

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // 1. Ambil header Authorization
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                AppError::Unauthorized("Header Authorization tidak ditemukan".to_string())
            })?;

        // 2. Ekstrak token dari "Bearer <token>"
        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            AppError::Unauthorized("Format harus: Bearer <token>".to_string())
        })?;

        // 3. Verifikasi JWT token
        let claims = jwt::verify_token(token, &state.config.jwt_secret).map_err(|_| {
            AppError::Unauthorized("Token tidak valid atau sudah expired".to_string())
        })?;

        // 4. Parse user_id dari claims.sub (string → UUID)
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            AppError::Unauthorized("Token claims tidak valid".to_string())
        })?;

        Ok(AuthUser { user_id })
    }
}
