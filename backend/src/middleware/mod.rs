// =============================================================================
// middleware/mod.rs — Middleware Registry
// =============================================================================
//
// Middleware adalah kode yang dijalankan SEBELUM atau SESUDAH handler.
// Di Axum, ada 2 cara membuat middleware:
//
//   1. Tower Layer  : Middleware level rendah (CORS, logging, timeout)
//                     → Sudah di-setup di lib.rs
//
//   2. Extractor    : Custom extractor yang bisa dipakai sebagai parameter handler
//                     → AuthUser di middleware/auth.rs
//
// Kita pakai pendekatan #2 untuk authentication karena lebih mudah
// dan idiomatic di Axum.
//
// 📝 CARA MENAMBAH MIDDLEWARE BARU:
//   1. Buat file baru (contoh: src/middleware/rate_limit.rs)
//   2. Daftarkan di sini: `pub mod rate_limit;`
//   3. Pilih pendekatan:
//      - Extractor (parameter handler) → implement FromRequestParts
//      - Layer (otomatis semua route) → implement Tower Layer/Service
// =============================================================================

pub mod auth;
