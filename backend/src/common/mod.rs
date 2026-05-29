// =============================================================================
// common/mod.rs — Shared Foundation Modules
// =============================================================================
//
// Folder `common/` mengelompokkan module-module "fondasi" yang dipakai
// di seluruh aplikasi:
//
//   config.rs  → Konfigurasi environment variables
//   db.rs      → Database connection pool & migrations
//   errors.rs  → Centralized error handling
//
// Kenapa dikelompokkan?
//   - Konsistensi: semua module punya folder
//   - Organisasi: pisahkan fondasi dari logika bisnis (handlers, models)
//   - Scalability: mudah menambah module fondasi baru
//
// 📝 CARA MENAMBAH MODULE FONDASI BARU:
//   1. Buat file baru di src/common/ (contoh: src/common/cache.rs)
//   2. Daftarkan di sini: `pub mod cache;`
//   3. Akses dari mana saja: `use crate::common::cache::...`
// =============================================================================

pub mod config;
pub mod db;
pub mod errors;
