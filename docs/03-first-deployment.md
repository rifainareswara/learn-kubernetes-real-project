# Fase 3: Deploy Pertama ke Kubernetes! 🚀

> **Tujuan**: Deploy aplikasi fullstack ke GKE untuk pertama kalinya.

## Apa yang Akan Kamu Pelajari
- Apply Kubernetes manifests secara berurutan
- Memahami setiap field di YAML manifest
- Debug pods yang gagal
- Port-forward untuk akses dari laptop

---

## Persiapan

Pastikan kamu sudah:
1. ✅ GKE cluster running (`kubectl get nodes`)
2. ✅ Images sudah di-push ke Artifact Registry
3. ✅ Update image path di deployment YAML

### Update Image Path

Sebelum deploy, kamu HARUS update image path di:
- `k8s/backend-deployment.yaml`
- `k8s/frontend-deployment.yaml`

```bash
# Ganti placeholder dengan path Artifact Registry kamu
# Contoh: asia-southeast2-docker.pkg.dev/my-project/taskmanager/backend:v1

# Cara cepat dengan sed:
export PROJECT_ID=your-project-id
export REGION=asia-southeast2

sed -i '' "s|REGION-docker.pkg.dev/PROJECT_ID|${REGION}-docker.pkg.dev/${PROJECT_ID}|g" \
  k8s/backend-deployment.yaml k8s/frontend-deployment.yaml
```

---

## Step 1: Apply Namespace

```bash
# Buat namespace "taskmanager"
kubectl apply -f k8s/namespace.yaml

# Verifikasi
kubectl get namespaces
```

Output:
```
NAME            STATUS   AGE
default         Active   1h
kube-system     Active   1h
taskmanager     Active   5s   ← Baru dibuat!
```

> 💡 Mulai sekarang, semua command tambahkan `-n taskmanager`

---

## Step 2: Apply ConfigMap & Secret

```bash
# ConfigMap (konfigurasi non-sensitif)
kubectl apply -f k8s/configmap.yaml

# Secret (data sensitif)
kubectl apply -f k8s/secret.yaml

# Verifikasi
kubectl get configmap -n taskmanager
kubectl get secret -n taskmanager
```

### Cek isi ConfigMap:
```bash
kubectl describe configmap taskmanager-config -n taskmanager
```

### Cek isi Secret (decode):
```bash
# Lihat nilai Secret (masih base64)
kubectl get secret taskmanager-secret -n taskmanager -o jsonpath='{.data.JWT_SECRET}'

# Decode base64
kubectl get secret taskmanager-secret -n taskmanager -o jsonpath='{.data.JWT_SECRET}' | base64 --decode
```

---

## Step 3: Deploy Database (PostgreSQL)

```bash
# Deploy PostgreSQL StatefulSet + Service
kubectl apply -f k8s/postgres-service.yaml
kubectl apply -f k8s/postgres-statefulset.yaml

# Tunggu sampai ready
kubectl rollout status statefulset/postgres -n taskmanager

# Cek pod
kubectl get pods -n taskmanager
```

Output yang diharapkan:
```
NAME         READY   STATUS    RESTARTS   AGE
postgres-0   1/1     Running   0          30s
```

### Debug kalau PostgreSQL gagal:
```bash
# Lihat events
kubectl describe pod postgres-0 -n taskmanager

# Lihat logs
kubectl logs postgres-0 -n taskmanager

# Cek PVC (Persistent Volume Claim)
kubectl get pvc -n taskmanager
```

---

## Step 4: Deploy Backend

```bash
# Deploy Backend Deployment + Service
kubectl apply -f k8s/backend-deployment.yaml
kubectl apply -f k8s/backend-service.yaml

# Tunggu sampai ready
kubectl rollout status deployment/backend -n taskmanager

# Cek pods
kubectl get pods -n taskmanager -l component=backend
```

Output yang diharapkan:
```
NAME                       READY   STATUS    RESTARTS   AGE
backend-7d8f9c6b4d-abc12   1/1     Running   0          30s
backend-7d8f9c6b4d-def34   1/1     Running   0          30s
```

### Test backend dari dalam cluster:

```bash
# Port forward ke laptop
kubectl port-forward svc/backend-service 3000:3000 -n taskmanager

# Di terminal lain:
curl http://localhost:3000/health
# Output: {"status":"ok","timestamp":"..."}

# Buka Swagger UI
open http://localhost:3000/swagger-ui/
```

### Debug kalau backend gagal:

```bash
# CrashLoopBackOff? Cek logs!
kubectl logs -l component=backend -n taskmanager

# Error umum:
# - "connection refused" → PostgreSQL belum ready
# - "env var not set" → ConfigMap/Secret belum di-apply
# - "migration error" → Database belum dibuat

# Masuk ke container untuk debug
kubectl exec -it deployment/backend -n taskmanager -- /bin/sh
```

---

## Step 5: Deploy Frontend

```bash
# Deploy Frontend Deployment + Service
kubectl apply -f k8s/frontend-deployment.yaml
kubectl apply -f k8s/frontend-service.yaml

# Tunggu
kubectl rollout status deployment/frontend -n taskmanager

# Cek pods
kubectl get pods -n taskmanager
```

### Test frontend:

```bash
# Port forward
kubectl port-forward svc/frontend-service 8080:80 -n taskmanager

# Buka browser
open http://localhost:8080
```

---

## Step 6: Lihat Semua Resources

```bash
# Lihat SEMUA yang sudah di-deploy
kubectl get all -n taskmanager
```

Output:
```
NAME                           READY   STATUS    RESTARTS   AGE
pod/backend-7d8f9c6b4d-abc12   1/1     Running   0          5m
pod/backend-7d8f9c6b4d-def34   1/1     Running   0          5m
pod/frontend-5c6d7e8f9g-hij56  1/1     Running   0          3m
pod/frontend-5c6d7e8f9g-klm78  1/1     Running   0          3m
pod/postgres-0                  1/1     Running   0          7m

NAME                       TYPE        CLUSTER-IP     PORT(S)
service/backend-service    ClusterIP   10.96.0.10     3000/TCP
service/frontend-service   ClusterIP   10.96.0.11     80/TCP
service/postgres-service   ClusterIP   None           5432/TCP

NAME                       READY   UP-TO-DATE   AVAILABLE   AGE
deployment.apps/backend    2/2     2            2           5m
deployment.apps/frontend   2/2     2            2           3m

NAME                             READY   AGE
statefulset.apps/postgres        1/1     7m
```

🎉 **Selamat! Aplikasi kamu sudah berjalan di Kubernetes!**

---

## Memahami YAML Manifest

Mari bedah `backend-deployment.yaml` baris per baris:

```yaml
apiVersion: apps/v1        # Versi API Kubernetes
kind: Deployment           # Tipe resource
metadata:
  name: backend            # Nama deployment
  namespace: taskmanager   # Di namespace mana
  labels:                  # Label untuk organizing
    app: taskmanager
    component: backend

spec:
  replicas: 2              # Jumlah pod yang diinginkan

  selector:                # "Deployment ini mengelola pod dengan label ini"
    matchLabels:
      app: taskmanager
      component: backend

  template:                # Template untuk membuat pod
    metadata:
      labels:              # Label pod HARUS match dengan selector di atas!
        app: taskmanager
        component: backend
    spec:
      containers:
        - name: backend
          image: .../backend:v1    # Docker image
          ports:
            - containerPort: 3000  # Port yang diexpose container
          env:                     # Environment variables
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:      # Ambil dari Secret
                  name: taskmanager-secret
                  key: DATABASE_URL
```

---

## Debugging Guide

### Pod Status yang Sering Ditemui

| Status | Artinya | Cara Fix |
|--------|---------|----------|
| `Running` | ✅ Berjalan normal | — |
| `Pending` | Menunggu scheduling | `kubectl describe pod` → cek Events |
| `ContainerCreating` | Sedang pull image/mount volume | Tunggu, cek events |
| `CrashLoopBackOff` | Container crash berulang | `kubectl logs` → cek error |
| `ImagePullBackOff` | Gagal pull Docker image | Cek image path, cek auth |
| `OOMKilled` | Kehabisan memory | Naikkan memory limits |

### Alur Debugging:

```
Pod tidak Running?
      │
      ├── kubectl describe pod <name> -n taskmanager
      │   └── Baca bagian "Events" di paling bawah
      │
      ├── kubectl logs <name> -n taskmanager
      │   └── Baca error message dari aplikasi
      │
      ├── kubectl get events -n taskmanager --sort-by=.lastTimestamp
      │   └── Lihat timeline events terbaru
      │
      └── kubectl exec -it <name> -n taskmanager -- /bin/sh
          └── Masuk ke container, debug dari dalam
```

---

## Eksperimen: Self-Healing

Coba hapus sebuah pod, dan lihat Kubernetes membuat pod baru otomatis!

```bash
# Lihat pods
kubectl get pods -n taskmanager -l component=backend

# Hapus satu pod
kubectl delete pod <pod-name> -n taskmanager

# Cepat! Lihat pod baru dibuat
kubectl get pods -n taskmanager -l component=backend --watch

# Pod baru muncul dalam hitungan detik! 🪄
```

---

## Eksperimen: Rolling Update

```bash
# Ganti image tag (simulasi deploy versi baru)
kubectl set image deployment/backend backend=NEW_IMAGE:v2 -n taskmanager

# Watch rolling update terjadi
kubectl rollout status deployment/backend -n taskmanager

# Lihat proses: pod baru dibuat → pod lama dihapus
kubectl get pods -n taskmanager -l component=backend --watch

# Rollback kalau ada masalah!
kubectl rollout undo deployment/backend -n taskmanager
```

---

## Checklist ✅

- [ ] Semua pods Running (`kubectl get pods -n taskmanager`)
- [ ] Backend health check OK via port-forward
- [ ] Frontend bisa diakses via port-forward
- [ ] Bisa register user dan login via frontend
- [ ] Bisa CRUD tasks via frontend
- [ ] Sudah coba self-healing (delete pod → auto recreate)
- [ ] Sudah coba rolling update + rollback
- [ ] Paham cara baca YAML manifest
- [ ] Paham alur debugging (describe → logs → exec)

---

**Selanjutnya**: [Fase 4 — Config & Secrets →](./04-config-secrets.md)
