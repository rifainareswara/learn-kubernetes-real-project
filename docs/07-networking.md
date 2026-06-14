# Fase 7: Networking — Ingress & Load Balancer

> **Tujuan**: Expose aplikasi ke internet menggunakan Ingress dan GKE Load Balancer.

## Sebelum Ingress vs Sesudah Ingress

```
TANPA Ingress:                    DENGAN Ingress:
                                  
Internet                          Internet
    │                                 │
    ✖ tidak bisa akses          ┌─────┴───────────────┐
                                │     INGRESS          │
Cluster:                        │                      │
┌─────────────────┐            │ /     → frontend    │
│ frontend (CIP)  │            │ /api  → backend     │
│ backend  (CIP)  │            └─────┬───────────────┘
│ postgres (CIP)  │                  │
└─────────────────┘            Cluster (bisa diakses!)
CIP = ClusterIP                
(hanya internal)                
```

---

## Tipe-Tipe Service

### ClusterIP (default)

```
Hanya bisa diakses dari DALAM cluster.
Pod lain bisa akses: backend-service:3000

Cocok untuk: komunikasi antar service internal
```

### NodePort

```
Internet → Node IP:30000-32767 → Service → Pod

Cocok untuk: testing/development, BUKAN production
Kelemahan: port range terbatas, tidak scalable
```

### LoadBalancer

```
Internet → Cloud Load Balancer → Service → Pod

Cocok untuk: 1 service yang perlu di-expose
Kelemahan: 1 LB per service = MAHAL ($18/bulan per LB!)
```

### Ingress (yang kita pakai!)

```
Internet → 1 Load Balancer → Ingress → Multiple Services

Cocok untuk: expose banyak service via 1 LB
Keunggulan: hemat biaya, path-based routing, TLS
```

---

## Apply Ingress

```bash
kubectl apply -f k8s/ingress.yaml

# Tunggu IP address (bisa 3-5 menit di GKE)
kubectl get ingress -n taskmanager --watch

# Dapatkan IP
export INGRESS_IP=$(kubectl get ingress taskmanager-ingress \
  -n taskmanager \
  -o jsonpath='{.status.loadBalancer.ingress[0].ip}')

echo "Aplikasi bisa diakses di: http://${INGRESS_IP}"
```

### Test Akses:

```bash
# Frontend
curl http://${INGRESS_IP}/

# Backend Health
curl http://${INGRESS_IP}/health

# API
curl http://${INGRESS_IP}/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"password123"}'

# Swagger UI
open http://${INGRESS_IP}/swagger-ui/
```

---

## Bagaimana Ingress Routing Bekerja

```yaml
# Di ingress.yaml:
rules:
  - http:
      paths:
        - path: /api         → backend-service:3000
        - path: /health       → backend-service:3000
        - path: /swagger-ui   → backend-service:3000
        - path: /              → frontend-service:80
```

```
Request masuk: http://35.xx.xx.xx/api/tasks
                                    ↑
                                    Ingress cocokkan path

Match: /api → kirim ke backend-service:3000
                        ↓
                   Load balance ke:
                   Pod backend-1 atau Pod backend-2
```

---

## DNS Setup (Opsional)

Jika kamu punya domain:

```bash
# 1. Dapatkan IP Ingress
kubectl get ingress -n taskmanager

# 2. Buat A record di DNS provider:
#    taskmanager.yourdomain.com → 35.xx.xx.xx

# 3. Update Ingress dengan hostname:
```

```yaml
# k8s/ingress.yaml (tambahkan host)
spec:
  rules:
    - host: taskmanager.yourdomain.com  # ← tambahkan ini
      http:
        paths:
          - path: /api
            ...
```

---

## TLS/HTTPS (Opsional)

GKE bisa otomatis manage SSL certificate:

```yaml
# Tambahkan annotations ke Ingress
metadata:
  annotations:
    networking.gke.io/managed-certificates: taskmanager-cert
    kubernetes.io/ingress.allow-http: "false"  # Force HTTPS

---
# Buat ManagedCertificate
apiVersion: networking.gke.io/v1
kind: ManagedCertificate
metadata:
  name: taskmanager-cert
  namespace: taskmanager
spec:
  domains:
    - taskmanager.yourdomain.com
```

---

## Debug Ingress

```bash
# Cek status Ingress
kubectl describe ingress taskmanager-ingress -n taskmanager

# Cek backend health dari Ingress perspective
# Di Events, cari "unhealthy" backends
kubectl get events -n taskmanager | grep -i ingress

# Cek apakah services punya endpoints (pods)
kubectl get endpoints -n taskmanager

# Common issues:
# - "backends are UNHEALTHY" → health probes gagal
# - No IP address → tunggu 3-5 menit
# - 404 Not Found → path routing salah
```

---

## Checklist ✅

- [ ] Paham perbedaan ClusterIP, NodePort, LoadBalancer, Ingress
- [ ] Ingress di-apply dan punya IP address
- [ ] Bisa akses frontend dari browser via Ingress IP
- [ ] Bisa akses API via Ingress IP
- [ ] Swagger UI bisa diakses dari internet
- [ ] Paham path-based routing

---

**Selanjutnya**: [Fase 8 — Observability →](./08-observability.md)
