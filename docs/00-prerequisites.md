# Fase 0: Prerequisites & Setup GCP

> **Tujuan**: Setup GCP project, buat GKE cluster, dan hubungkan `kubectl` ke cluster.

## Apa yang Akan Kamu Pelajari
- Cara setup project di Google Cloud Platform
- Cara membuat Kubernetes cluster (GKE Autopilot)
- Cara menghubungkan `kubectl` lokal ke GKE
- Cara setup Artifact Registry (tempat simpan Docker images)

---

## Step 1: Cek Tools yang Sudah Terinstall

Pastikan tools berikut sudah terinstall di laptop kamu:

```bash
# Cek Docker
docker --version

# Cek kubectl (Kubernetes CLI)
kubectl version --client

# Cek Google Cloud SDK
gcloud --version
```

✅ Semua harus menghasilkan output versi, bukan error.

---

## Step 2: Login ke Google Cloud

```bash
# Login ke akun Google kamu
gcloud auth login

# Set default project (GANTI dengan project ID kamu)
gcloud config set project YOUR_PROJECT_ID

# Verifikasi
gcloud config get-value project
```

> 💡 **Project ID** bisa kamu lihat di [console.cloud.google.com](https://console.cloud.google.com)

---

## Step 3: Aktifkan APIs yang Dibutuhkan

```bash
# Aktifkan Kubernetes Engine API
gcloud services enable container.googleapis.com

# Aktifkan Artifact Registry API (untuk simpan Docker images)
gcloud services enable artifactregistry.googleapis.com

# Aktifkan Cloud Build API (opsional, untuk build di cloud)
gcloud services enable cloudbuild.googleapis.com
```

---

## Step 4: Buat Artifact Registry Repository

Artifact Registry adalah "Docker Hub milik Google" — tempat menyimpan Docker images.

```bash
# Buat repository
gcloud artifacts repositories create taskmanager \
  --repository-format=docker \
  --location=asia-southeast2 \
  --description="Docker images untuk Task Manager K8s project"

# Konfigurasi Docker agar bisa push ke Artifact Registry
gcloud auth configure-docker asia-southeast2-docker.pkg.dev

# Verifikasi
gcloud artifacts repositories list --location=asia-southeast2
```

> 💡 `asia-southeast2` = Jakarta. Pilih region terdekat untuk latency rendah.

---

## Step 5: Buat GKE Autopilot Cluster

```bash
# Buat cluster Autopilot
# Autopilot = Google mengelola nodes, kamu hanya fokus deploy aplikasi
gcloud container clusters create-auto taskmanager-cluster \
  --region=asia-southeast2 \
  --project=YOUR_PROJECT_ID

# ⏳ Proses ini butuh 5-10 menit...
```

### Kenapa Autopilot?

| Fitur | Autopilot | Standard |
|-------|-----------|----------|
| Kelola nodes | Google | Kamu sendiri |
| Bayar | Per pod (sesuai pemakaian) | Per node (24/7) |
| Setup | Sangat mudah | Perlu konfigurasi |
| Cocok untuk | Belajar, production kecil | Production besar |
| Estimasi biaya | ~$3-5/hari | ~$5-15/hari |

---

## Step 6: Hubungkan kubectl ke GKE

```bash
# Get credentials (otomatis konfigurasi kubectl)
gcloud container clusters get-credentials taskmanager-cluster \
  --region=asia-southeast2

# Verifikasi koneksi
kubectl cluster-info

# Lihat nodes
kubectl get nodes
```

Output yang diharapkan:
```
Kubernetes control plane is running at https://xx.xx.xx.xx
...

NAME                                          STATUS   ROLES    AGE   VERSION
gk3-taskmanager-cluster-pool-xxxx-xxxx        Ready    <none>   5m    v1.xx.x
```

🎉 **Selamat!** kubectl kamu sudah terhubung ke GKE!

---

## Step 7: Kenalan Pertama dengan kubectl

Coba jalankan beberapa perintah dasar:

```bash
# Lihat semua namespaces
kubectl get namespaces

# Lihat pods di semua namespace
kubectl get pods --all-namespaces

# Lihat detail cluster
kubectl cluster-info dump | head -50
```

---

## ⚠️ Tips Hemat Credit

```bash
# MATIKAN cluster saat tidak dipakai (hemat $3-5/hari!)
gcloud container clusters delete taskmanager-cluster \
  --region=asia-southeast2 \
  --quiet

# Buat ulang saat mau belajar lagi
gcloud container clusters create-auto taskmanager-cluster \
  --region=asia-southeast2
```

> 🔴 **PENTING**: Selalu matikan cluster saat selesai belajar!
> Trial credit $300 bisa habis dalam 2 bulan kalau cluster nyala terus 24/7.
> Dengan mematikan cluster saat tidak dipakai, credit bisa bertahan 3-6 bulan.

---

## Checklist ✅

- [x] `gcloud auth login` berhasil
- [x] Project di-set dengan `gcloud config set project`
- [x] APIs diaktifkan (container, artifactregistry)
- [x] Artifact Registry repository dibuat
- [x] GKE cluster dibuat dan running
- [x] `kubectl get nodes` menampilkan nodes
- [x] Paham cara matikan cluster untuk hemat credit

---

**Selanjutnya**: [Fase 1 — Konsep Dasar Kubernetes →](./01-konsep-dasar.md)
