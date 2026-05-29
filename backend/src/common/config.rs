// =============================================================================
// config.rs — Konfigurasi Aplikasi
// =============================================================================
//
// FASE 3: Database Integration
//
// Modul ini memuat konfigurasi dari environment variables.
// Kenapa pakai environment variables?
//   - Aman: secret tidak disimpan di source code
//   - Fleksibel: beda environment (dev/staging/prod) bisa pakai nilai berbeda
//   - Standard: mengikuti 12-Factor App methodology
//
// File .env dibaca oleh `dotenvy` di main.rs sebelum Config::from_env() dipanggil.
//
// 📝 CARA MENAMBAH KONFIGURASI BARU:
//   1. Tambah field di struct Config
//   2. Baca dari env var di from_env()
//   3. Tambah contohnya di .env.example
//   4. Akses via state.config.field_name di handler
// =============================================================================

use std::env;

/// Konfigurasi aplikasi yang dimuat dari environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Connection string PostgreSQL.
    /// Format: postgres://user:password@host:port/database
    pub database_url: String,

    /// Secret key untuk sign/verify JWT token.
    /// ⚠️  HARUS diganti di production! Minimal 32 karakter random.
    pub jwt_secret: String,

    /// Alamat bind server (default: "0.0.0.0" = semua interface)
    pub host: String,

    /// Port server (default: 3000)
    pub port: u16,
}

impl Config {
    /// Memuat konfigurasi dari environment variables.
    ///
    /// # Panics
    /// Panic jika `DATABASE_URL` atau `JWT_SECRET` tidak di-set.
    ///
    /// # Contoh Environment Variables
    /// ```env
    /// DATABASE_URL=postgres://postgres:password@localhost:5432/learn_axum
    /// JWT_SECRET=super-secret-key
    /// HOST=0.0.0.0
    /// PORT=3000
    /// ```
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("❌ DATABASE_URL harus di-set di .env"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("❌ JWT_SECRET harus di-set di .env"),
            // unwrap_or_else: jika env var tidak ada, pakai nilai default
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("❌ PORT harus berupa angka"),
        }
    }

    /// Mengembalikan alamat lengkap server (contoh: "0.0.0.0:3000").
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
