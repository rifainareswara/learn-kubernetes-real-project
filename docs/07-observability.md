# Fase 7: Observability — Monitoring & Logging

> **Tujuan**: Memahami health probes, resource management, dan cara monitoring di GKE.

## Health Probes — Tiga Pertanyaan Kubernetes

Kubernetes punya 3 jenis probe untuk mengetahui kondisi container:

```
┌──────────────────────────────────────────────────────────────┐
│ Container Lifecycle:                                          │
│                                                                │
│  Start ──────────────────────────────────────────── Running    │
│    │                                                    │      │
│    ▼                                                    ▼      │
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐     │
│ │ STARTUP      │  │ READINESS    │  │ LIVENESS          │     │
│ │ Probe        │  │ Probe        │  │ Probe             │     │
│ │              │  │              │  │                    │     │
│ │ "Sudah       │  │ "Siap terima │  │ "Masih hidup?"    │     │
│ │  selesai     │  │  traffic?"   │  │                    │     │
│ │  startup?"   │  │              │  │ Gagal → RESTART    │     │
│ │              │  │ Gagal → STOP │  │                    │     │
│ │ Gagal →      │  │ kirim traffic│  │                    │     │
│ │ RESTART      │  │              │  │                    │     │
│ └──────────────┘  └──────────────┘  └──────────────────┘     │
│       ↑                                                        │
│  Selama startup probe                                         │
│  belum sukses, liveness                                       │
│  & readiness TIDAK aktif                                      │
└──────────────────────────────────────────────────────────────┘
```

### Kapan Masing-Masing Dibutuhkan?

| Probe | Kapan Dibutuhkan | Contoh |
|-------|-----------------|--------|
| **Startup** | App butuh waktu lama startup | Database migration, compile |
| **Readiness** | App butuh waktu siap setelah start | Connect ke DB, load cache |
| **Liveness** | Deteksi app yang freeze/deadlock | Memory leak, infinite loop |

### Probe di Project Kita

```yaml
# Di backend-deployment.yaml:

# 1. Startup: "Apakah sudah selesai startup?"
startupProbe:
  httpGet:
    path: /health      # Endpoint yang dicek
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 5     # Cek setiap 5 detik
  failureThreshold: 30 # 30 kali gagal = restart (total 150 detik)

# 2. Readiness: "Siap terima traffic?"
readinessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 5
  failureThreshold: 3

# 3. Liveness: "Masih hidup?"
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 15
  failureThreshold: 3
```

---

## Resource Management

### Requests vs Limits

```
┌─────────────────────────────────────────────────┐
│ Node (4 CPU, 8GB RAM)                            │
│                                                   │
│  ┌─────────────────────────────────────────────┐ │
│  │ Pod: backend                                 │ │
│  │                                               │ │
│  │  requests:        limits:                     │ │
│  │    cpu: 100m       cpu: 500m                  │ │
│  │    memory: 128Mi   memory: 256Mi              │ │
│  │                                               │ │
│  │  ┌──────────┬──────────┐                     │ │
│  │  │ DIJAMIN  │ BOLEH    │                     │ │
│  │  │ (100m)   │ pakai    │                     │ │
│  │  │          │ sampai   │                     │ │
│  │  │          │ (500m)   │                     │ │
│  │  └──────────┴──────────┘                     │ │
│  └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘

requests = minimum yang DIJAMIN
  → Scheduler pakai ini untuk pilih node

limits = maximum yang BOLEH dipakai
  → CPU: throttle (diperlambat)
  → Memory: OOMKilled (pod di-restart)
```

### Ukuran CPU dan Memory

```
CPU:
  1 CPU = 1000m (millicores)
  100m  = 0.1 CPU (10%)
  500m  = 0.5 CPU (50%)
  2000m = 2 CPU

Memory:
  Mi = Mebibytes (1 Mi = 1,048,576 bytes)
  64Mi  = ~67 MB
  128Mi = ~134 MB
  256Mi = ~268 MB
  1Gi   = ~1.07 GB
```

---

## Monitoring di GKE Console

### Akses GKE Dashboard:

1. Buka [console.cloud.google.com](https://console.cloud.google.com)
2. Navigation → **Kubernetes Engine** → **Workloads**
3. Klik pada deployment untuk melihat detail

### Hal yang Bisa Dilihat:

- **CPU Usage**: Seberapa banyak CPU yang dipakai
- **Memory Usage**: Penggunaan RAM
- **Pod Status**: Healthy, unhealthy, restarting
- **Events**: Apa yang terjadi (scaling, restart, error)
- **Logs**: Output dari container

### Lihat Metrics via kubectl:

```bash
# CPU dan Memory usage semua pods
kubectl top pods -n taskmanager

# CPU dan Memory usage nodes
kubectl top nodes

# Output contoh:
# NAME                    CPU(cores)   MEMORY(bytes)
# backend-xxx-abc12       15m          89Mi
# backend-xxx-def34       12m          85Mi
# frontend-xxx-ghi56      2m           12Mi
# postgres-0              25m          120Mi
```

---

## Structured Logging

Backend kita menggunakan `tracing` crate (Rust) untuk structured logging.

### Lihat Logs:

```bash
# Logs semua backend pods
kubectl logs -l component=backend -n taskmanager --tail=50

# Follow (live) logs
kubectl logs -l component=backend -n taskmanager -f

# Logs dari pod tertentu
kubectl logs backend-xxx-abc12 -n taskmanager

# Logs dari container yang crash sebelumnya
kubectl logs backend-xxx-abc12 -n taskmanager --previous
```

### Log Levels (diatur via ConfigMap):

```yaml
# ConfigMap: RUST_LOG
RUST_LOG: "taskmanager_api=debug,tower_http=debug"

# Levels (dari paling verbose ke paling sedikit):
#   trace  → SEMUA (termasuk internal)
#   debug  → Info debug (request details)
#   info   → Informasi umum (default production)
#   warn   → Peringatan
#   error  → Error saja
```

---

## Eksperimen: Apa yang Terjadi Saat Probe Gagal?

### Simulasi Readiness Probe Failure:

```bash
# 1. Watch pods secara real-time
kubectl get pods -n taskmanager -l component=backend --watch

# 2. Di terminal lain, kill health endpoint (misal, isi database salah)
# Atau scale database ke 0 sementara:
kubectl scale statefulset postgres -n taskmanager --replicas=0

# 3. Perhatikan backend pods:
#    READY: 1/1 → 0/1 (readiness probe gagal!)
#    Kubernetes STOP kirim traffic ke pod ini

# 4. Kembalikan database
kubectl scale statefulset postgres -n taskmanager --replicas=1

# 5. Pods kembali Ready: 0/1 → 1/1
```

---

## Eksperimen: OOMKilled

```bash
# Coba set memory limit sangat kecil
kubectl patch deployment backend -n taskmanager --type=json \
  -p='[{"op":"replace","path":"/spec/template/spec/containers/0/resources/limits/memory","value":"32Mi"}]'

# Pod akan OOMKilled!
kubectl get pods -n taskmanager --watch

# Cek Events
kubectl describe pod <pod-name> -n taskmanager | grep -A5 "State:"
# State: Waiting
#   Reason: CrashLoopBackOff
# Last State: Terminated
#   Reason: OOMKilled  ← Kehabisan memory!

# Kembalikan ke normal
kubectl apply -f k8s/backend-deployment.yaml
```

---

## Checklist ✅

- [ ] Paham 3 jenis probe: Startup, Readiness, Liveness
- [ ] Paham perbedaan requests vs limits
- [ ] Bisa pakai `kubectl top pods` untuk melihat resource usage
- [ ] Bisa baca logs dengan `kubectl logs`
- [ ] Sudah coba eksperimen readiness probe failure
- [ ] Paham apa itu OOMKilled
- [ ] Tahu cara akses GKE monitoring dashboard

---

**Selanjutnya**: [Fase 8 — Production Best Practices →](./08-production.md)
