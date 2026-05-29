// =============================================================================
// db.rs — Database Connection & Migrations
// =============================================================================
//
// FASE 3: Database Integration
//
// Modul ini mengelola koneksi ke PostgreSQL menggunakan SQLx.
//
// Konsep penting:
//   - Connection Pool: Kumpulan koneksi database yang di-reuse.
//     Membuat koneksi baru itu mahal (butuh TCP handshake, auth, dll).
//     Pool menyimpan beberapa koneksi yang siap pakai.
//
//   - Migrations: File SQL yang dijalankan secara berurutan untuk
//     membuat/mengubah struktur database (tabel, index, dll).
//     Memastikan database selalu sinkron dengan kode.
//
// 📝 CARA MENAMBAH MIGRATION BARU:
//   1. Buat file SQL baru di migrations/ dengan format:
//      {YYYYMMDDHHMMSS}_{deskripsi}.sql
//      Contoh: 20260529010000_add_tags_table.sql
//   2. Tulis SQL CREATE TABLE / ALTER TABLE di file tersebut
//   3. Restart server — migration akan otomatis dijalankan
// =============================================================================

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Membuat connection pool ke PostgreSQL.
///
/// Pool ini akan di-share ke semua handler melalui `AppState`.
///
/// # Arguments
/// - `database_url`: Connection string (dari .env)
///
/// # Konfigurasi Pool
/// - `max_connections(10)`: Maksimal 10 koneksi simultan
/// - `acquire_timeout(30s)`: Timeout 30 detik jika semua koneksi sedang dipakai
///
/// # Contoh
/// ```rust
/// let pool = create_pool("postgres://localhost/mydb").await?;
/// ```
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(database_url)
        .await?;

    tracing::info!("✅ Database connection pool created");
    Ok(pool)
}

/// Menjalankan semua migration yang belum dieksekusi.
///
/// SQLx menyimpan daftar migration yang sudah dijalankan di tabel
/// `_sqlx_migrations`. Jadi migration yang sama tidak akan dijalankan 2x.
///
/// # Panics
/// Panic jika migration gagal (misalnya SQL syntax error).
pub async fn run_migrations(pool: &PgPool) {
    tracing::info!("🔄 Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("❌ Database migration failed");
    tracing::info!("✅ Database migrations completed");
}
