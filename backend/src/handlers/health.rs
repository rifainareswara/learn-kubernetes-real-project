// =============================================================================
// handlers/health.rs — Health Check Endpoint
// =============================================================================
//
// FASE 2: Routing & Handler
//
// Endpoint health check yang paling sederhana. Berguna untuk:
//   - Monitoring (uptime check)
//   - Load balancer health probe
//   - Kubernetes liveness/readiness probe
//   - Verifikasi bahwa server berjalan
//
// 💡 TIP: Ini adalah contoh handler PALING SEDERHANA di Axum.
//         Perhatikan pattern-nya:
//         1. Fungsi async
//         2. Return type yang implement IntoResponse (Json<T>)
//         3. Tidak perlu parameter jika tidak butuh data dari request
// =============================================================================

use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// Response body untuk health check.
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// Status server: "healthy"
    pub status: String,
    /// Nama service
    pub service: String,
    /// Versi aplikasi (dari Cargo.toml)
    pub version: String,
}

/// Health check endpoint.
///
/// Mengembalikan status server beserta versi aplikasi.
/// Endpoint ini tidak membutuhkan authentication.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Server is healthy", body = HealthResponse)
    )
)]
pub async fn health_check() -> Json<HealthResponse> {
    // env!("CARGO_PKG_VERSION") dibaca dari Cargo.toml saat compile time
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "learn-rust-axum".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
