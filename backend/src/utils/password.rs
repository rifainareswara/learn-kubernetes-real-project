// =============================================================================
// utils/password.rs — Password Hashing & Verification
// =============================================================================
//
// FASE 5: Authentication
//
// Modul ini menangani hashing dan verifikasi password menggunakan Argon2.
//
// ⚠️  JANGAN PERNAH simpan password sebagai plain text!
//     Selalu hash password sebelum disimpan ke database.
//
// Kenapa Argon2?
//   - Pemenang Password Hashing Competition (2015)
//   - Lebih aman dari bcrypt dan scrypt
//   - Tahan terhadap GPU cracking dan side-channel attacks
//   - Direkomendasikan oleh OWASP
//
// Cara kerja:
//   1. hash_password("mypassword")
//      → "$argon2id$v=19$m=19456,t=2,p=1$salt$hash"
//      (format: algorithm$version$params$salt$hash)
//
//   2. verify_password("mypassword", "$argon2id$...")
//      → true (cocok) / false (tidak cocok)
//
// 📝 CARA MEMBUAT FUNGSI UTILITY BARU:
//   Perhatikan pattern di bawah:
//   1. Definisikan fungsi dengan `pub fn` (publik) atau `fn` (privat)
//   2. Tentukan parameter dan return type
//   3. Gunakan `Result<T, E>` jika fungsi bisa gagal
//   4. Dokumentasikan dengan `///` doc comments
// =============================================================================

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash password menggunakan Argon2id.
///
/// # Arguments
/// - `password`: Password plain text dari user
///
/// # Returns
/// String hash yang siap disimpan ke database.
///
/// # Contoh
/// ```rust
/// let hash = hash_password("my_secure_password")?;
/// // hash = "$argon2id$v=19$m=19456,t=2,p=1$random_salt$computed_hash"
/// ```
///
/// # Errors
/// Gagal jika ada masalah dengan random number generator.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    // 1. Generate random salt (garam) — unik untuk setiap password
    //    Salt mencegah rainbow table attack
    let salt = SaltString::generate(&mut OsRng);

    // 2. Buat instance Argon2 dengan parameter default
    //    (memory: 19MB, iterations: 2, parallelism: 1)
    let argon2 = Argon2::default();

    // 3. Hash password + salt → menghasilkan hash string
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;

    // 4. Konversi ke string format yang bisa disimpan
    Ok(hash.to_string())
}

/// Verifikasi password terhadap hash yang tersimpan.
///
/// # Arguments
/// - `password`: Password plain text yang diinput user saat login
/// - `hash`: Hash yang tersimpan di database
///
/// # Returns
/// `true` jika password cocok, `false` jika tidak.
///
/// # Contoh
/// ```rust
/// let hash = hash_password("my_password")?;
/// assert!(verify_password("my_password", &hash)?);   // ✅ cocok
/// assert!(!verify_password("wrong_pass", &hash)?);    // ❌ tidak cocok
/// ```
pub fn verify_password(
    password: &str,
    hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    // 1. Parse hash string kembali ke struct PasswordHash
    let parsed_hash = PasswordHash::new(hash)?;

    // 2. Verifikasi: bandingkan password input dengan hash
    //    .is_ok() mengubah Result → bool (true jika Ok, false jika Err)
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
