# Fase 5: Database di Kubernetes

> **Tujuan**: Memahami StatefulSet, PersistentVolume, dan cara mengelola database di Kubernetes.

## Kenapa Database Butuh Perlakuan Khusus?

```
Backend (Stateless):          Database (Stateful):
┌──────────┐                  ┌──────────┐
│ Pod A    │  ← mati?         │ Pod A    │  ← mati?
│ backend  │  → buat baru,    │ postgres │  → DATA-nya harus SELAMAT!
│          │     tidak masalah │          │     
└──────────┘                  └──────────┘
```

**Stateless app** (backend, frontend):
- Tidak simpan data di container
- Pod bisa di-replace kapan saja tanpa kehilangan apa-apa
- Cocok pakai **Deployment**

**Stateful app** (database, message queue):
- Data HARUS survive restart, delete, dan reschedule
- Pod butuh identitas TETAP
- Cocok pakai **StatefulSet** + **PersistentVolume**

---

## StatefulSet vs Deployment

| Fitur | Deployment | StatefulSet |
|-------|-----------|-------------|
| Nama pod | Random (`backend-7d8f-abc12`) | Terurut (`postgres-0`, `postgres-1`) |
| Startup order | Semua sekaligus | Berurutan (0 → 1 → 2) |
| Delete order | Semua sekaligus | Terbalik (2 → 1 → 0) |
| Storage | Ephemeral (hilang saat restart) | Persistent (survive restart) |
| DNS | Pakai Service IP | Setiap pod punya DNS unik |
| Use case | API, frontend, worker | Database, cache, queue |

---

## PersistentVolume (PV) & PersistentVolumeClaim (PVC)

```
┌─────────────────────────────────────────────────────────┐
│                    GKE Cluster                            │
│                                                           │
│  ┌────────────────┐       ┌──────────────────────┐       │
│  │ Pod: postgres-0│       │ Google Persistent     │       │
│  │                │ mount │ Disk (10GB)           │       │
│  │ /var/lib/      │◄─────┤                       │       │
│  │ postgresql/data│       │ Data survives:        │       │
│  │                │       │  ✅ Pod restart       │       │
│  └────────────────┘       │  ✅ Pod delete        │       │
│         ▲                 │  ✅ Node failure      │       │
│         │                 └──────────────────────┘       │
│  ┌──────┴──────┐                    ▲                     │
│  │    PVC      │                    │                     │
│  │ "Saya butuh │ ──── bind ────────┘                     │
│  │  5GB disk"  │                                          │
│  └─────────────┘                                          │
└─────────────────────────────────────────────────────────┘

PV  = Disk fisik (Google Persistent Disk)
PVC = Request untuk disk ("saya butuh 5GB")
Pod = Mount disk ke path tertentu
```

### Dynamic Provisioning di GKE

Di GKE, kamu TIDAK perlu membuat PV manual. Cukup buat PVC, dan GKE otomatis:
1. Buat Google Persistent Disk
2. Buat PV
3. Bind PV ke PVC
4. Mount ke pod

Ini berkat **StorageClass**:

```bash
# Lihat StorageClass yang tersedia di GKE
kubectl get storageclass

# Output:
# NAME              PROVISIONER             AGE
# standard-rwo      pd.csi.storage.gke.io   1d   ← Default
# premium-rwo       pd.csi.storage.gke.io   1d   ← SSD (lebih cepat)
```

---

## Cek PostgreSQL yang Sudah Di-deploy

```bash
# Lihat StatefulSet
kubectl get statefulset -n taskmanager

# Lihat PVC (claim storage)
kubectl get pvc -n taskmanager

# Lihat PV (volume fisik yang dibuat otomatis)
kubectl get pv

# Detail PVC
kubectl describe pvc postgres-data-postgres-0 -n taskmanager
```

---

## Eksperimen: Data Survive Restart

```bash
# 1. Buat task via API
kubectl port-forward svc/backend-service 3000:3000 -n taskmanager &

curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"test","email":"test@test.com","password":"password123"}'

# 2. Delete pod PostgreSQL!
kubectl delete pod postgres-0 -n taskmanager

# 3. Tunggu pod baru dibuat
kubectl get pods -n taskmanager --watch

# 4. Cek data masih ada!
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"password123"}'

# ✅ Login masih berhasil! Data tidak hilang!
```

---

## Backup Database

```bash
# Masuk ke pod PostgreSQL
kubectl exec -it postgres-0 -n taskmanager -- bash

# Backup database ke file
pg_dump -U postgres taskmanager > /tmp/backup.sql

# Exit dari pod
exit

# Copy backup ke laptop
kubectl cp taskmanager/postgres-0:/tmp/backup.sql ./backup.sql
```

### Restore dari Backup

```bash
# Copy file ke pod
kubectl cp ./backup.sql taskmanager/postgres-0:/tmp/backup.sql

# Restore
kubectl exec -it postgres-0 -n taskmanager -- \
  psql -U postgres -d taskmanager -f /tmp/backup.sql
```

---

## Kapan Pakai StatefulSet vs Cloud SQL?

| Kriteria | StatefulSet | Cloud SQL |
|----------|-------------|-----------|
| Biaya | Lebih murah | Lebih mahal (~$25/bulan+) |
| Maintenance | Kamu kelola sendiri | Google kelola |
| Backup | Manual | Otomatis |
| HA/Replication | Setup manual | 1 klik |
| Cocok untuk | Belajar, dev | Production |

**Untuk belajar**: StatefulSet (sudah kita pakai).
**Untuk production**: Cloud SQL (lebih reliable, auto-backup, auto-failover).

---

## Checklist ✅

- [ ] Paham perbedaan StatefulSet vs Deployment
- [ ] Paham PV, PVC, dan StorageClass
- [ ] PostgreSQL berjalan sebagai StatefulSet
- [ ] Sudah coba eksperimen: delete pod → data survive
- [ ] Paham cara backup dan restore database
- [ ] Paham kapan pakai StatefulSet vs Cloud SQL

---

**Selanjutnya**: [Fase 6 — Networking →](./06-networking.md)
