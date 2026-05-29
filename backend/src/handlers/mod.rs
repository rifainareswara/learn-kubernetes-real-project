// =============================================================================
// handlers/mod.rs — Handler Module Registry
// =============================================================================
//
// Handler adalah fungsi yang memproses HTTP request dan mengembalikan response.
// Di Axum, handler adalah fungsi async biasa dengan signature khusus:
//
//   async fn handler(
//       State(state): State<Arc<AppState>>,  ← shared state (DB, config)
//       auth: AuthUser,                       ← auth middleware (opsional)
//       Path(id): Path<Uuid>,                 ← path parameter
//       Query(params): Query<ListParams>,     ← query string
//       Json(body): Json<CreateRequest>,       ← request body
//   ) -> Result<Json<Response>, AppError> {    ← return type
//       // ... logika bisnis ...
//   }
//
// Parameter-parameter ini disebut "Extractors". Axum otomatis:
//   1. Parse request sesuai tipe extractor
//   2. Inject hasilnya ke parameter handler
//   3. Return error jika parsing gagal
//
// 📝 CARA MENAMBAH HANDLER BARU:
//   1. Buat file baru (contoh: src/handlers/category.rs)
//   2. Daftarkan di sini: `pub mod category;`
//   3. Tulis fungsi handler dengan parameter extractor yang dibutuhkan
//   4. Daftarkan route-nya di src/routes/
// =============================================================================

pub mod auth;
pub mod health;
pub mod task;
