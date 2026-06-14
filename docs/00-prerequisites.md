# Fase 0: Prerequisites & Setup Environment (Minikube / GCP)

> **Tujuan**: Menyiapkan environment Kubernetes lokal (menggunakan Minikube) atau di Cloud (menggunakan Google Kubernetes Engine / GKE) dan menghubungkan `kubectl`.

Untuk belajar, kamu bisa memilih salah satu jalur setup di bawah ini:
1. **Jalur A: Minikube (Lokal & 100% Gratis)** 💡 *Sangat direkomendasikan untuk pemula agar hemat kuota internet dan bebas biaya cloud.*
2. **Jalur B: Google Cloud Platform / GKE (Cloud)** ☁️ *Untuk simulasi lingkungan produksi yang sesungguhnya di GCP.*

---

## 🛠️ Step 1: Install Tools Dasar

Apapun jalur yang kamu pilih, pastikan tool dasar ini sudah terinstall di laptop kamu:

```bash
# Cek Docker (Minikube & GCP butuh ini)
docker --version

# Cek kubectl (Kubernetes CLI)
kubectl version --client
```

---

## 💻 JALUR A: Setup Menggunakan Minikube (Lokal)

Minikube adalah tool yang menjalankan cluster Kubernetes single-node di dalam Virtual Machine (VM) atau container Docker di laptop kamu.

### 1. Install Minikube
Jika belum punya, install minikube terlebih dahulu:
*   **macOS (Homebrew)**: `brew install minikube`
*   **Windows (Chocolatey)**: `choco install minikube`
*   *Atau download installer langsung dari [situs resmi Minikube](https://minikube.sigs.k8s.io/docs/start/).*

### 2. Jalankan Minikube
```bash
# Jalankan cluster Minikube menggunakan driver Docker
minikube start --driver=docker

# Jika laptop kamu menggunakan Apple Silicon (M1/M2/M3), jalankan:
# minikube start --driver=docker --arch=arm64
```

> ⏳ Proses pertama kali akan mendownload image Kubernetes (sekitar 1-2 GB). Silakan tunggu hingga selesai.

### 3. Verifikasi Koneksi
Minikube secara otomatis akan mengonfigurasi `kubectl` kamu agar terhubung ke cluster lokal ini.
```bash
# Cek info cluster
kubectl cluster-info

# Lihat node minikube yang sedang berjalan
kubectl get nodes
```
Output yang diharapkan:
```
Kubernetes control plane is running at https://127.0.0.1:XXXXX
CoreDNS is running at https://127.0.0.1:XXXXX/...

NAME       STATUS   ROLES           AGE   VERSION
minikube   Ready    control-plane   1m    v1.xx.x
```

### 4. Tips Minikube: Point Docker ke Minikube Daemon (PENTING!)
Agar kamu bisa membuild Docker Image lokal tanpa perlu melakukan push ke Docker Hub/Artifact Registry, kamu bisa mengarahkan Docker CLI laptopmu langsung ke dalam Docker Daemon di dalam Minikube:
```bash
# Jalankan ini di terminal (khusus macOS/Linux)
eval $(minikube docker-env)

# Untuk Windows (PowerShell):
# & minikube -p minikube docker-env | Invoke-Expression
```
> ⚠️ **Catatan**: Perintah `eval` ini hanya berlaku untuk session terminal saat ini. Jika kamu membuka tab terminal baru, kamu harus menjalankannya lagi.

---

## ☁️ JALUR B: Setup Menggunakan GCP & GKE (Cloud)

Jika kamu ingin mencoba langsung di cloud provider menggunakan Google Kubernetes Engine (GKE):

### 1. Install Google Cloud CLI (gcloud)
Pastikan gcloud CLI sudah terinstall:
```bash
gcloud --version
```

### 2. Login ke Google Cloud
```bash
# Login ke akun Google kamu
gcloud auth login

# Set default project (GANTI dengan project ID kamu)
gcloud config set project YOUR_PROJECT_ID

# Verifikasi
gcloud config get-value project
```
> 🔄 *Tips: Jika kamu memiliki banyak akun gcloud di laptop, beralihlah dengan perintah `gcloud config set account EMAIL_KAMU@domain.com`.*

### 3. Aktifkan APIs yang Dibutuhkan
```bash
# Aktifkan Kubernetes Engine API & Artifact Registry
gcloud services enable container.googleapis.com
gcloud services enable artifactregistry.googleapis.com
```

### 4. Buat Artifact Registry Repository (Docker Registry di GCP)
```bash
# Buat repository
gcloud artifacts repositories create taskmanager \
  --repository-format=docker \
  --location=asia-southeast2 \
  --description="Docker images untuk Task Manager K8s project"

# Konfigurasi Docker agar bisa push ke Artifact Registry
gcloud auth configure-docker asia-southeast2-docker.pkg.dev
```

### 5. Buat GKE Autopilot Cluster
```bash
# Buat cluster Autopilot (di region Jakarta: asia-southeast2)
gcloud container clusters create-auto taskmanager-cluster \
  --region=asia-southeast2 \
  --project=YOUR_PROJECT_ID
```

### 6. Hubungkan kubectl ke GKE
```bash
gcloud container clusters get-credentials taskmanager-cluster \
  --region=asia-southeast2
```

---

## 🎯 Step 3: Kenalan Pertama dengan kubectl

Apapun jalur yang kamu pilih (Minikube maupun GCP), coba jalankan perintah-perintah ini untuk memastikan kubectl bekerja:

```bash
# Lihat semua namespaces yang ada secara default
kubectl get namespaces

# Lihat pods yang sedang berjalan di cluster (awal-awal biasanya masih kosong atau sistem saja)
kubectl get pods --all-namespaces
```

---

## ⚠️ Menghemat Resource & Credit

Agar laptop tidak lemot atau saldo Google Cloud tidak habis sia-sia saat tidak belajar:

### A. Jika Pakai Minikube (Hemat RAM/CPU Laptop)
```bash
# Hentikan Minikube saat selesai belajar (status tersimpan)
minikube stop

# Jalankan kembali kapan saja dengan:
minikube start

# Hapus cluster sepenuhnya jika ingin membersihkan resource:
minikube delete
```

### B. Jika Pakai GKE (Hemat Credit GCP)
```bash
# Hapus cluster agar tidak ditagih terus menerus ($3-5 per hari!)
gcloud container clusters delete taskmanager-cluster \
  --region=asia-southeast2 \
  --quiet
```

---

## Checklist ✅

- [ ] Driver Containerization (Docker) sudah berjalan.
- [ ] Salah satu cluster siap (`minikube status` atau `gcloud container clusters list`).
- [ ] `kubectl get nodes` berhasil menampilkan minimal 1 node berstatus `Ready`.

---

**Selanjutnya**: [Fase 1 — Konsep Dasar Kubernetes →](./01-konsep-dasar.md)
