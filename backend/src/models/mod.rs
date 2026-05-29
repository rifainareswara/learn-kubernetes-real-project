// =============================================================================
// models/mod.rs — Data Model Registry
// =============================================================================
//
// Folder `models/` berisi struct yang merepresentasikan data di database
// dan juga DTO (Data Transfer Object) untuk request/response.
//
// Pattern yang dipakai:
//   - Model    : Struct yang 1:1 mapping dengan tabel database
//   - Request  : Struct untuk menerima data dari client (Deserialize)
//   - Response : Struct untuk mengirim data ke client (Serialize)
//
// Kenapa pisahkan Model, Request, Response?
//   - Keamanan: password_hash tidak pernah dikirim ke client
//   - Fleksibel: field request bisa berbeda dari field di DB
//   - Validasi: hanya field yang dibutuhkan yang diterima
//
// 📝 CARA MENAMBAH MODEL BARU:
//   1. Buat file baru (contoh: src/models/category.rs)
//   2. Daftarkan di sini: `pub mod category;`
//   3. Buat struct Model (derive: sqlx::FromRow, Serialize)
//   4. Buat struct Request/Response (derive: Deserialize/Serialize)
//   5. Implementasikan database query functions
// =============================================================================

pub mod task;
pub mod user;
