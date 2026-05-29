// =============================================================================
// utils/jwt.rs — JWT Token Management
// =============================================================================
//
// FASE 5: Authentication
//
// JWT (JSON Web Token) digunakan untuk autentikasi stateless:
//   1. User login → server buat JWT token → kirim ke client
//   2. Client simpan token (di header Authorization)
//   3. Setiap request, client kirim token → server verifikasi
//
// Struktur JWT:
//   Header.Payload.Signature
//   eyJhbGci...  ←  Base64 encoded, BUKAN encrypted!
//
// Payload (Claims) berisi:
//   - sub: Subject (user ID)
//   - iat: Issued At (waktu token dibuat)
//   - exp: Expiration (waktu token kedaluwarsa)
//
// ⚠️  JWT BUKAN enkripsi! Siapa pun bisa membaca isinya.
//     Signature hanya menjamin token tidak dimodifikasi.
//
// 📝 CARA MENAMBAH CLAIM BARU:
//   1. Tambah field di struct Claims (contoh: `pub role: String`)
//   2. Set nilainya di create_token()
//   3. Akses di middleware/auth.rs setelah verify_token()
// =============================================================================

use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Claims yang disimpan di dalam JWT token.
///
/// Ini adalah "payload" dari JWT — data yang akan di-encode.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: User ID (sebagai string UUID)
    pub sub: String,

    /// Issued At: waktu token dibuat (Unix timestamp)
    pub iat: usize,

    /// Expiration: waktu token kedaluwarsa (Unix timestamp)
    pub exp: usize,
}

/// Durasi token berlaku (24 jam dalam detik).
const TOKEN_EXPIRY_SECONDS: i64 = 24 * 60 * 60;

/// Membuat JWT token baru untuk user yang berhasil login.
///
/// # Arguments
/// - `user_id`: UUID user yang login
/// - `secret`: JWT secret key dari konfigurasi
///
/// # Returns
/// String JWT token yang siap dikirim ke client.
///
/// # Contoh
/// ```rust
/// let token = create_token(user.id, &config.jwt_secret)?;
/// // token = "eyJhbGciOiJIUzI1NiIs..."
/// ```
pub fn create_token(
    user_id: Uuid,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now().timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        iat: now as usize,
        exp: (now + TOKEN_EXPIRY_SECONDS) as usize,
    };

    // encode() = Header + Claims + Signature → JWT string
    // Header::default() menggunakan algoritma HS256
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Memverifikasi dan decode JWT token.
///
/// # Arguments
/// - `token`: JWT token string dari Authorization header
/// - `secret`: JWT secret key untuk verifikasi signature
///
/// # Returns
/// `Claims` jika token valid dan belum expired.
///
/// # Errors
/// - Token expired
/// - Signature tidak valid (token dimodifikasi)
/// - Format token salah
pub fn verify_token(
    token: &str,
    secret: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        // Validation::default() otomatis check: exp, algorithm (HS256)
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
