# Fase 2: Containerization — Docker Images

> **Tujuan**: Build Docker images untuk backend (Rust) dan frontend (Svelte), test lokal dengan Docker Compose, dan push ke Artifact Registry.

## Apa yang Akan Kamu Pelajari
- Multi-stage Docker build (image kecil dan aman)
- Docker Compose untuk local development
- Push image ke Google Artifact Registry

---

## Step 1: Pahami Dockerfile Backend (Rust)

Buka file `backend/Dockerfile`. Ini menggunakan **multi-stage build**:

```
Stage 1: BUILDER                    Stage 2: RUNTIME
┌──────────────────────┐           ┌──────────────────────┐
│ FROM rust:slim        │           │ FROM debian:slim      │
│                       │           │                       │
│ ┌───────────────────┐│           │ ┌───────────────────┐│
│ │ Cargo.toml        ││    COPY   │ │ taskmanager-api    ││ ← Binary saja!
│ │ src/              ││ ────────▶ │ │ migrations/        ││ ← Migrations
│ │ target/           ││  binary   │ │                     ││
│ │ (2GB+ total)      ││           │ │ (~80MB total)       ││
│ └───────────────────┘│           │ └───────────────────┘│
└──────────────────────┘           └──────────────────────┘
   Dibuang setelah build              Image FINAL yang kecil!
```

**Kenapa multi-stage?**
- Single-stage: image ~2GB (berisi compiler, source code, dll)
- Multi-stage: image ~80MB (hanya binary + minimal runtime)
- Lebih kecil = lebih cepat pull, lebih sedikit attack surface

### Trik: Cache Dependencies

```dockerfile
# Copy Cargo.toml DULU (tanpa source code)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release  ← Build dependencies (di-CACHE!)

# Baru copy source code
COPY src/ src/
RUN cargo build --release  ← Hanya compile source code kamu
```

Tanpa trik ini, setiap kali kamu edit 1 baris code, Docker harus download dan compile SEMUA dependencies dari awal (~5-10 menit).

Dengan trik ini, kalau hanya source code berubah, dependencies pakai cache (~30 detik).

---

## Step 2: Pahami Dockerfile Frontend (Svelte)

Buka file `frontend/Dockerfile`:

```
Stage 1: BUILD                     Stage 2: SERVE
┌──────────────────────┐           ┌──────────────────────┐
│ FROM node:lts-alpine  │           │ FROM nginx:alpine     │
│                       │           │                       │
│ npm ci                │    COPY   │ /usr/share/nginx/html │
│ npm run build         │ ────────▶ │   ├── index.html     │
│                       │  /dist    │   ├── assets/         │
│ node_modules (300MB)  │           │   └── ...             │
│ src/ (development)    │           │                       │
│                       │           │ (~25MB total)          │
└──────────────────────┘           └──────────────────────┘
```

---

## Step 3: Test Lokal dengan Docker Compose

Sebelum deploy ke Kubernetes, test dulu secara lokal:

```bash
# Masuk ke folder project
cd learn-kubernetes-real-project

# Build dan jalankan semua services
docker compose up --build

# Atau jalankan di background
docker compose up --build -d
```

### Cek apakah berjalan:

```bash
# Lihat status containers
docker compose ps

# Lihat logs
docker compose logs -f

# Lihat logs service tertentu
docker compose logs -f backend
```

### Test endpoints:

```bash
# Health check
curl http://localhost:3000/health

# Akses frontend
open http://localhost:8080

# Swagger UI
open http://localhost:3000/swagger-ui/
```

### Bersihkan:

```bash
# Stop semua
docker compose down

# Stop + hapus data database
docker compose down -v
```

---

## Step 4: Build Images untuk Production

```bash
# Build backend image
docker build -t taskmanager-backend:v1 ./backend

# Build frontend image
docker build -t taskmanager-frontend:v1 ./frontend

# Cek ukuran image
docker images | grep taskmanager
```

Output yang diharapkan:
```
REPOSITORY              TAG    SIZE
taskmanager-backend     v1     ~80-100MB  ✅ (bukan 2GB!)
taskmanager-frontend    v1     ~25-40MB   ✅
```

---

## Step 5: Push ke Artifact Registry

```bash
# Set variabel (GANTI dengan milik kamu!)
export PROJECT_ID=your-project-id
export REGION=asia-southeast2
export REGISTRY=${REGION}-docker.pkg.dev/${PROJECT_ID}/taskmanager

# Tag images
docker tag taskmanager-backend:v1 ${REGISTRY}/backend:v1
docker tag taskmanager-backend:v1 ${REGISTRY}/backend:latest
docker tag taskmanager-frontend:v1 ${REGISTRY}/frontend:v1
docker tag taskmanager-frontend:v1 ${REGISTRY}/frontend:latest

# Push ke Artifact Registry
docker push ${REGISTRY}/backend:v1
docker push ${REGISTRY}/backend:latest
docker push ${REGISTRY}/frontend:v1
docker push ${REGISTRY}/frontend:latest

# Verifikasi
gcloud artifacts docker images list ${REGISTRY}
```

---

## Konsep Penting: Image Tags

```
registry/image:TAG
                 ↑
                 ini penting!

Best practices:
  ✅ backend:v1.0.0          (semantic versioning)
  ✅ backend:abc123f          (git commit SHA)
  ✅ backend:2024-01-15       (tanggal)
  ⚠️ backend:latest          (ambigu — versi mana?)
  ❌ backend (tanpa tag)      (default ke :latest)
```

Kenapa hindari `latest` di production?
- Tidak jelas versi mana yang berjalan
- Kalau ada masalah, susah rollback
- Kubernetes mungkin cache image lama

---

## Troubleshooting

### Docker build gagal (Rust)
```bash
# Error: "could not compile"
# → Cek Cargo.toml, pastikan dependencies benar

# Error: "out of disk space"
# → Bersihkan Docker cache
docker system prune -a
```

### Docker Compose: backend tidak bisa connect ke PostgreSQL
```bash
# Pastikan PostgreSQL sudah ready sebelum backend start
# docker-compose.yml sudah menggunakan healthcheck + depends_on

# Cek logs PostgreSQL
docker compose logs postgres

# Restart backend saja
docker compose restart backend
```

---

## Checklist ✅

- [ ] Paham multi-stage Docker build
- [ ] `docker compose up --build` berjalan tanpa error
- [ ] Bisa akses frontend di http://localhost:8080
- [ ] Bisa akses backend health di http://localhost:3000/health
- [ ] Bisa register dan login via frontend atau curl
- [ ] Images sudah di-push ke Artifact Registry
- [ ] Paham pentingnya image tags

---

**Selanjutnya**: [Fase 3 — Deploy Pertama ke Kubernetes →](./03-first-deployment.md)
