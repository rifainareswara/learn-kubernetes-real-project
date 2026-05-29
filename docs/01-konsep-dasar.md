# Fase 1: Konsep Dasar Kubernetes

> **Tujuan**: Paham arsitektur Kubernetes, core objects, dan perintah-perintah dasar `kubectl`.

## Apa itu Kubernetes?

**Kubernetes** (K8s) adalah **container orchestrator** — sistem yang mengelola container secara otomatis.

### Kenapa Butuh Kubernetes?

Bayangkan kamu punya aplikasi yang jalan di Docker:

```
Tanpa Kubernetes:
  - Server 1: docker run backend   ← Kalau crash, siapa yang restart?
  - Server 2: docker run backend   ← Gimana load balance?
  - Server 3: docker run frontend  ← Gimana update tanpa downtime?
  - Server 4: docker run postgres  ← Gimana backup otomatis?

Dengan Kubernetes:
  "Saya mau 3 backend, 2 frontend, 1 database"
  → Kubernetes yang urus semuanya:
    ✅ Auto-restart kalau crash
    ✅ Load balancing otomatis
    ✅ Rolling update (zero downtime)
    ✅ Auto-scaling saat traffic tinggi
    ✅ Self-healing (pod mati → buat baru)
```

---

## Arsitektur Kubernetes

```
┌─────────────────────────────────────────────────────────────────┐
│                        KUBERNETES CLUSTER                        │
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                     CONTROL PLANE                            │ │
│  │  (Otak dari Kubernetes — kamu TIDAK perlu kelola ini di GKE) │ │
│  │                                                               │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │ │
│  │  │  API Server   │  │  Scheduler   │  │  Controller      │   │ │
│  │  │  (kubectl     │  │  (pilih node │  │  Manager          │   │ │
│  │  │   bicara ke   │  │   untuk pod) │  │  (jaga desired    │   │ │
│  │  │   sini)       │  │              │  │   state)          │   │ │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘   │ │
│  │  ┌──────────────┐                                             │ │
│  │  │  etcd         │  ← Database untuk semua data cluster       │ │
│  │  │  (key-value)  │                                             │ │
│  │  └──────────────┘                                             │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                                                                   │
│  ┌──────────────────┐  ┌──────────────────┐                     │
│  │   WORKER NODE 1   │  │   WORKER NODE 2   │  ← Tempat pod     │
│  │                    │  │                    │     berjalan       │
│  │  ┌──────────────┐ │  │  ┌──────────────┐ │                    │
│  │  │  kubelet      │ │  │  │  kubelet      │ │ ← Agent yang      │
│  │  │  (agent)      │ │  │  │  (agent)      │ │   jalankan pod    │
│  │  └──────────────┘ │  │  └──────────────┘ │                    │
│  │                    │  │                    │                    │
│  │  ┌─────┐ ┌─────┐ │  │  ┌─────┐ ┌─────┐ │                    │
│  │  │Pod 1│ │Pod 2│ │  │  │Pod 3│ │Pod 4│ │                    │
│  │  └─────┘ └─────┘ │  │  └─────┘ └─────┘ │                    │
│  └──────────────────┘  └──────────────────┘                     │
└─────────────────────────────────────────────────────────────────┘
```

### Komponen Penting:

| Komponen | Analogi | Fungsi |
|----------|---------|--------|
| **Cluster** | Kantor | Seluruh infrastruktur Kubernetes |
| **Control Plane** | Manager kantor | Mengambil keputusan, menjadwalkan kerja |
| **Worker Node** | Meja kerja | Tempat pod (container) berjalan |
| **kubelet** | Asisten di meja | Agent yang menjalankan instruksi |
| **Pod** | Karyawan | Unit terkecil, berisi 1+ container |

---

## Core Objects Kubernetes

### 1. Pod — Unit Terkecil

```
┌─────────────────────────┐
│         POD               │
│                           │
│  ┌─────────────────────┐ │
│  │     Container        │ │  ← Biasanya 1 container per pod
│  │  (backend:v1)        │ │
│  └─────────────────────┘ │
│                           │
│  IP: 10.0.0.5            │  ← Setiap pod punya IP sendiri
│  Status: Running          │
└─────────────────────────┘
```

**Analogi**: Pod = satu karyawan di kantor.

Key points:
- Pod adalah unit **terkecil** yang bisa di-deploy
- Pod bersifat **ephemeral** (sementara) — bisa mati kapan saja
- Biasanya 1 container per pod
- Setiap pod punya IP address sendiri
- **Jangan buat pod langsung!** Gunakan Deployment.

### 2. Deployment — Mengelola Pod

```
┌──────────────────────────────────────────┐
│            DEPLOYMENT                      │
│            replicas: 3                     │
│                                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │  Pod 1   │ │  Pod 2   │ │  Pod 3   │  │
│  │  v1      │ │  v1      │ │  v1      │  │
│  └──────────┘ └──────────┘ └──────────┘  │
│                                            │
│  Pod 2 crash! → Kubernetes buat Pod baru:  │
│                                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐  │
│  │  Pod 1   │ │  Pod 4   │ │  Pod 3   │  │
│  │  v1      │ │  v1 ✨   │ │  v1      │  │
│  └──────────┘ └──────────┘ └──────────┘  │
└──────────────────────────────────────────┘
```

**Analogi**: Deployment = manager yang memastikan jumlah karyawan selalu cukup.

Key points:
- Kamu bilang "saya mau 3 replicas"
- Kubernetes **selalu** memastikan ada 3 pod berjalan
- Kalau pod mati → otomatis buat baru (**self-healing**)
- Update image → **rolling update** (zero downtime)

### 3. Service — Networking Internal

```
┌──────────────────────────────────────────────────┐
│  Frontend Pod                                     │
│  "Saya mau akses backend"                        │
│  → Tapi IP pod berubah setiap restart!            │
│  → Solusi: akses via Service name                 │
└────────────────────┬─────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────┐
│  Service: backend-service                         │
│  IP: 10.96.0.10 (STABIL — tidak berubah!)        │
│  DNS: backend-service.taskmanager.svc.cluster.local│
│                                                    │
│  Load Balance ke:                                  │
│    → Pod 1 (10.0.0.5)                             │
│    → Pod 2 (10.0.0.6)                             │
│    → Pod 3 (10.0.0.7)                             │
└──────────────────────────────────────────────────┘
```

**Analogi**: Service = nomor telepon kantor. Karyawan bisa ganti, tapi nomor telepon tetap.

Tipe-tipe Service:

| Tipe | Akses | Use Case |
|------|-------|----------|
| **ClusterIP** | Hanya dari dalam cluster | Backend ↔ Database |
| **NodePort** | Dari luar via node IP:port | Testing/development |
| **LoadBalancer** | Dari internet via LB | Production |

### 4. ConfigMap & Secret

```
ConfigMap (data PUBLIK):
  HOST: "0.0.0.0"
  PORT: "3000"
  RUST_LOG: "info"

Secret (data RAHASIA):
  DATABASE_URL: "postgres://..."  (base64 encoded)
  JWT_SECRET: "super-secret..."   (base64 encoded)

         ↓ Inject ke container sebagai env vars ↓

Pod:
  env:
    HOST = "0.0.0.0"       ← dari ConfigMap
    JWT_SECRET = "super..."  ← dari Secret
```

### 5. Namespace — Isolasi Resource

```
Cluster
├── namespace: default        ← jangan pakai ini
├── namespace: kube-system    ← system pods Kubernetes
├── namespace: taskmanager    ← project kita! 🎯
└── namespace: monitoring     ← tools monitoring (opsional)
```

**Analogi**: Namespace = departemen di kantor. Setiap departemen punya resource sendiri.

### 6. Ingress — Pintu Masuk dari Internet

```
Internet
    │
    ▼
┌──────────────────────────────────┐
│  Ingress                          │
│                                    │
│  /          → frontend-service    │
│  /api/*     → backend-service     │
│  /swagger-* → backend-service     │
└──────────────────────────────────┘
```

**Analogi**: Ingress = resepsionis. Tamu datang, diarahkan ke departemen yang tepat.

---

## Deklaratif vs Imperatif

Kubernetes menggunakan pendekatan **deklaratif**:

```
Imperatif (command):
  "Buat pod bernama backend dengan image backend:v1"
  kubectl run backend --image=backend:v1

Deklaratif (YAML):
  "Ini yang saya mau. Kubernetes, wujudkan."
  kubectl apply -f deployment.yaml
```

**Kenapa deklaratif lebih baik?**
- YAML bisa di-version control (Git)
- Reproducible (bisa di-recreate kapan saja)
- Self-documenting (baca YAML = tahu konfigurasi)
- Bisa di-review sebelum apply

---

## Cheatsheet kubectl

### Melihat Resource

```bash
# Format umum: kubectl get <resource> -n <namespace>

kubectl get pods -n taskmanager          # List pods
kubectl get pods -n taskmanager -o wide  # Dengan info extra (IP, node)
kubectl get deployments -n taskmanager   # List deployments
kubectl get services -n taskmanager      # List services
kubectl get all -n taskmanager           # Semua resource
```

### Detail & Debug

```bash
# Describe = info lengkap + events (PALING BERGUNA untuk debug!)
kubectl describe pod <pod-name> -n taskmanager

# Logs container
kubectl logs <pod-name> -n taskmanager
kubectl logs <pod-name> -n taskmanager --tail=100 -f  # Follow (live)
kubectl logs -l component=backend -n taskmanager       # By label

# Masuk ke dalam container (seperti docker exec)
kubectl exec -it <pod-name> -n taskmanager -- /bin/sh
```

### Apply & Delete

```bash
# Apply manifest (buat atau update)
kubectl apply -f k8s/deployment.yaml

# Apply semua file di folder
kubectl apply -f k8s/

# Delete resource
kubectl delete -f k8s/deployment.yaml
kubectl delete pod <pod-name> -n taskmanager
```

### Scaling

```bash
# Scale manual
kubectl scale deployment backend -n taskmanager --replicas=5

# Lihat rollout status
kubectl rollout status deployment/backend -n taskmanager

# Rollback ke versi sebelumnya
kubectl rollout undo deployment/backend -n taskmanager
```

### Port Forward (akses dari laptop)

```bash
# Forward port dari pod ke laptop
kubectl port-forward pod/<pod-name> 3000:3000 -n taskmanager

# Forward dari service
kubectl port-forward svc/backend-service 3000:3000 -n taskmanager

# Sekarang bisa akses: http://localhost:3000
```

---

## Quiz Check-Pemahaman

Jawab pertanyaan ini di kepala sebelum lanjut ke fase berikutnya:

1. **Apa bedanya Pod dan Container?**
   <details>
   <summary>Jawaban</summary>
   Pod adalah wrapper yang berisi 1 atau lebih container. Pod punya IP sendiri. Container di dalam pod share network dan storage.
   </details>

2. **Kenapa tidak buat Pod langsung, tapi pakai Deployment?**
   <details>
   <summary>Jawaban</summary>
   Karena Deployment mengelola lifecycle pod: auto-restart, scaling, rolling update. Kalau buat pod langsung, pod mati = mati selamanya.
   </details>

3. **Apa bedanya ConfigMap dan Secret?**
   <details>
   <summary>Jawaban</summary>
   ConfigMap untuk data non-sensitif (bisa dilihat siapa saja). Secret untuk data sensitif (di-encode base64, akses dibatasi RBAC).
   </details>

4. **Kalau `kubectl get pods` menunjukkan pod status `CrashLoopBackOff`, apa yang harus dilakukan?**
   <details>
   <summary>Jawaban</summary>
   Cek logs: `kubectl logs <pod-name>` dan describe: `kubectl describe pod <pod-name>`. CrashLoopBackOff artinya container crash → restart → crash → restart berulang.
   </details>

5. **Apa itu rolling update?**
   <details>
   <summary>Jawaban</summary>
   Saat update image, Kubernetes buat pod baru (v2) dulu, pastikan ready, baru hapus pod lama (v1). Sehingga tidak ada downtime.
   </details>

---

## Checklist ✅

- [ ] Paham arsitektur K8s (Control Plane vs Worker Node)
- [ ] Paham 6 core objects: Pod, Deployment, Service, ConfigMap, Secret, Ingress
- [ ] Paham kenapa deklaratif (YAML) lebih baik dari imperatif
- [ ] Hafal perintah kubectl dasar (get, describe, logs, apply)
- [ ] Bisa jawab 5 quiz di atas

---

**Selanjutnya**: [Fase 2 — Containerization →](./02-containerization.md)
