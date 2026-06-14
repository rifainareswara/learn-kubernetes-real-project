# Fase 9: Production Best Practices

> **Tujuan**: Autoscaling, keamanan jaringan, CI/CD, dan tips optimasi biaya.

## HPA — Horizontal Pod Autoscaler

### Bagaimana HPA Bekerja

```
Traffic rendah:          Traffic tinggi:          Traffic turun:
┌──────────────┐        ┌──────────────┐        ┌──────────────┐
│ 2 pods       │        │ 6 pods       │        │ 2 pods       │
│ CPU: 20%     │  ──▶   │ CPU: 65%     │  ──▶   │ CPU: 25%     │
│              │        │              │        │              │
│ HPA: "OK,    │        │ HPA: "Masih  │        │ HPA: "Terlalu│
│  cukup 2"    │        │  di bawah    │        │  banyak, saya│
│              │        │  70%, OK"    │        │  kurangi"    │
└──────────────┘        └──────────────┘        └──────────────┘
```

### Apply HPA

```bash
kubectl apply -f k8s/hpa.yaml

# Monitor HPA
kubectl get hpa -n taskmanager

# Output:
# NAME          REFERENCE            TARGETS         MINPODS  MAXPODS  REPLICAS
# backend-hpa   Deployment/backend   15%/70%         2        10       2
#                                    ↑   ↑
#                                  actual target

# Watch secara real-time
kubectl get hpa -n taskmanager --watch
```

### Test Autoscaling

```bash
# 1. Buat load test (banyak request)
# Install hey (HTTP load generator)
# brew install hey

# 2. Kirim 1000 request ke backend
hey -n 1000 -c 50 http://${INGRESS_IP}/health

# 3. Watch HPA scale up
kubectl get hpa -n taskmanager --watch
kubectl get pods -n taskmanager -l component=backend --watch

# 4. Setelah traffic turun, pods akan scale down (setelah 5 menit stabilization)
```

---

## Network Policy — Keamanan Jaringan

### Tanpa Network Policy (Default)

```
SEMUA pod bisa bicara ke SEMUA pod:

Frontend ──→ PostgreSQL   ← BAHAYA! Frontend seharusnya tidak akses DB langsung!
Backend  ──→ Frontend     ← Tidak perlu
Random Pod ──→ PostgreSQL ← SANGAT BAHAYA!
```

### Dengan Network Policy

```
Frontend ──→ Backend ──→ PostgreSQL
   │              │           │
   └── HANYA ─────┘   HANYA──┘
      ini boleh        dari backend

Random Pod ──✖──→ PostgreSQL  ← DIBLOKIR!
Frontend ──✖──→ PostgreSQL    ← DIBLOKIR!
```

### Apply Network Policy

```bash
kubectl apply -f k8s/network-policy.yaml

# Verifikasi
kubectl get networkpolicy -n taskmanager

# Test: coba akses PostgreSQL dari frontend pod (harusnya GAGAL!)
kubectl exec -it deployment/frontend -n taskmanager -- \
  sh -c "apk add --no-cache postgresql-client && psql -h postgres-service -U postgres"
# Harusnya timeout/connection refused!
```

---

## CI/CD dengan GitHub Actions

### Alur CI/CD

```
Developer push ke main
         │
         ▼
┌─────────────────┐
│ GitHub Actions   │
│                  │
│ 1. Build images  │
│ 2. Push to       │
│    Artifact      │
│    Registry      │
│ 3. Deploy to     │
│    GKE           │
└─────────────────┘
         │
         ▼
   GKE Cluster
   (auto-updated!)
```

### Setup

1. **Buat Service Account di GCP:**

```bash
# Buat service account
gcloud iam service-accounts create github-deployer \
  --display-name="GitHub Actions Deployer"

# Berikan permission
export PROJECT_ID=your-project-id
export SA_EMAIL=github-deployer@${PROJECT_ID}.iam.gserviceaccount.com

gcloud projects add-iam-policy-binding ${PROJECT_ID} \
  --member="serviceAccount:${SA_EMAIL}" \
  --role="roles/container.developer"

gcloud projects add-iam-policy-binding ${PROJECT_ID} \
  --member="serviceAccount:${SA_EMAIL}" \
  --role="roles/artifactregistry.writer"

# Buat key JSON
gcloud iam service-accounts keys create github-sa-key.json \
  --iam-account=${SA_EMAIL}
```

2. **Set GitHub Secrets:**

Di repository GitHub → Settings → Secrets → Actions:

| Secret | Value |
|--------|-------|
| `GCP_PROJECT_ID` | ID project GCP kamu |
| `GCP_SA_KEY` | Isi file `github-sa-key.json` |
| `GKE_CLUSTER_NAME` | `taskmanager-cluster` |
| `GKE_CLUSTER_ZONE` | `asia-southeast2` |

3. **Push dan lihat pipeline berjalan!**

```bash
git add .
git commit -m "Deploy to GKE"
git push origin main

# Lihat progress di GitHub → Actions tab
```

---

## Cost Optimization (Hemat Credit!)

### 1. Matikan Cluster Saat Tidak Dipakai

```bash
# Simpan manifests dulu
kubectl get all -n taskmanager -o yaml > backup-all.yaml

# Hapus cluster
gcloud container clusters delete taskmanager-cluster \
  --region=asia-southeast2 --quiet

# Buat ulang saat butuh
gcloud container clusters create-auto taskmanager-cluster \
  --region=asia-southeast2

# Restore
kubectl apply -f k8s/
```

### 2. Gunakan Preemptible/Spot VMs

Spot VMs 60-91% lebih murah! Tapi bisa diambil kapan saja oleh Google.

```bash
# Untuk Standard cluster (bukan Autopilot):
gcloud container node-pools create spot-pool \
  --cluster=taskmanager-cluster \
  --spot \
  --num-nodes=2
```

### 3. Right-Size Resources

```bash
# Lihat actual usage
kubectl top pods -n taskmanager

# Bandingkan dengan requests/limits
kubectl get pods -n taskmanager -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.spec.containers[0].resources}{"\n"}{end}'

# Kalau actual CPU 10m tapi request 100m → turunkan request
```

### 4. Estimasi Biaya

| Resource | Biaya/hari (Autopilot) |
|----------|----------------------|
| Backend (2 pods, 100m CPU, 128Mi) | ~$0.50 |
| Frontend (2 pods, 50m CPU, 64Mi) | ~$0.25 |
| PostgreSQL (1 pod, 100m CPU, 256Mi) | ~$0.40 |
| Ingress (Load Balancer) | ~$0.60 |
| PV (5GB SSD) | ~$0.01 |
| **Total** | **~$1.76/hari** |

Dengan trial credit $300: **cukup untuk ~170 hari** (kalau nyala terus).
Kalau dimatikan saat tidak dipakai: **bisa 6+ bulan!**

---

## Ringkasan: Apa yang Sudah Kamu Pelajari

```
Fase 0: ✅ Setup GCP & GKE
Fase 1: ✅ Konsep dasar (Pod, Deployment, Service, dll)
Fase 2: ✅ Docker multi-stage build, Docker Compose
Fase 3: ✅ Run lokal di minikube (Kustomize overlay)
Fase 4: ✅ Deploy pertama ke K8s, debugging
Fase 5: ✅ ConfigMap & Secret management
Fase 6: ✅ StatefulSet & Persistent Volume (database)
Fase 7: ✅ Ingress & Load Balancer (networking)
Fase 8: ✅ Health probes, resource limits, monitoring
Fase 9: ✅ HPA, Network Policy, CI/CD
```

### Skills yang Kamu Miliki Sekarang:

- ☸️ Deploy fullstack app ke Kubernetes
- 🐳 Build optimized Docker images
- 🔒 Manage secrets dan config secara aman
- 📊 Monitor dan debug aplikasi di K8s
- 🔄 Setup CI/CD pipeline
- 📈 Autoscaling berdasarkan traffic
- 🛡️ Keamanan jaringan antar pod

---

## Langkah Selanjutnya

Setelah menguasai dasar-dasar ini, kamu bisa eksplorasi:

1. **Helm** — Package manager untuk Kubernetes (templating YAML)
2. **Kustomize** — Customization tanpa templating
3. **Istio/Linkerd** — Service mesh (advanced networking)
4. **ArgoCD** — GitOps continuous delivery
5. **Prometheus + Grafana** — Monitoring & alerting advanced
6. **Terraform** — Infrastructure as Code (buat cluster via code)
7. **CKAD Certification** — Sertifikasi Kubernetes resmi

---

## Checklist ✅

- [ ] HPA di-apply dan bisa scale berdasarkan CPU
- [ ] Network Policy di-apply
- [ ] Paham alur CI/CD dengan GitHub Actions
- [ ] Paham cara hemat GCP trial credit
- [ ] Bisa deploy ulang seluruh stack dari nol

---

🎉 **Selamat! Kamu sudah belajar Kubernetes dari nol sampai production!**
