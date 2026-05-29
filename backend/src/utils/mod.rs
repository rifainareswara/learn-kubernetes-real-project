// =============================================================================
// utils/mod.rs — Utility Module Registry
// =============================================================================
//
// File `mod.rs` di sebuah folder berfungsi sebagai "pintu masuk" module.
// Semua sub-module harus didaftarkan di sini agar bisa diakses.
//
// 📝 CARA MENAMBAH UTILITY BARU:
//   1. Buat file baru di src/utils/ (contoh: src/utils/email.rs)
//   2. Tambahkan `pub mod email;` di bawah ini
//   3. Akses dari mana saja: `use crate::utils::email::send_email;`
// =============================================================================

pub mod jwt;
pub mod password;
