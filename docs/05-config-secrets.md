# Fase 5: Config & Secrets

> **Tujuan**: Memahami cara mengelola konfigurasi dan data sensitif di Kubernetes.

## Kenapa Tidak Hardcode di Deployment?

```yaml
# ❌ BURUK — hardcode di deployment
env:
  - name: DATABASE_URL
    value: "postgres://user:password123@db:5432/mydb"
  - name: JWT_SECRET
    value: "my-secret-key"

# ✅ BAGUS — ambil dari ConfigMap & Secret
env:
  - name: DATABASE_URL
    valueFrom:
      secretKeyRef:
        name: taskmanager-secret
        key: DATABASE_URL
```

**Alasannya:**
1. **Separation of Concerns** — konfigurasi terpisah dari deployment
2. **Keamanan** — Secret di-encode dan akses dibatasi RBAC
3. **Reusability** — banyak pod bisa share ConfigMap/Secret yang sama
4. **Easy Update** — ubah config tanpa rebuild image

---

## ConfigMap Deep Dive

### Cara 1: envFrom (inject semua key)

```yaml
# Semua key di ConfigMap jadi env var
envFrom:
  - configMapRef:
      name: taskmanager-config

# HOST="0.0.0.0", PORT="3000", RUST_LOG="info"
# Semua otomatis tersedia!
```

### Cara 2: valueFrom (inject key tertentu)

```yaml
# Hanya key tertentu
env:
  - name: APP_PORT    # Nama env var di container
    valueFrom:
      configMapKeyRef:
        name: taskmanager-config
        key: PORT       # Key di ConfigMap
```

### Cara 3: Volume Mount (sebagai file)

```yaml
# Mount ConfigMap sebagai file
volumeMounts:
  - name: config-volume
    mountPath: /etc/config     # Path di container
volumes:
  - name: config-volume
    configMap:
      name: taskmanager-config
# Hasilnya: /etc/config/HOST, /etc/config/PORT, dll
```

### Membuat ConfigMap via CLI:

```bash
# Dari literal values
kubectl create configmap my-config \
  --from-literal=KEY1=value1 \
  --from-literal=KEY2=value2 \
  -n taskmanager

# Dari file
kubectl create configmap nginx-config \
  --from-file=nginx.conf \
  -n taskmanager

# Dari env file
kubectl create configmap app-config \
  --from-env-file=.env \
  -n taskmanager
```

---

## Secret Deep Dive

### Encode/Decode Base64:

```bash
# Encode
echo -n "my-password" | base64
# Output: bXktcGFzc3dvcmQ=

# Decode
echo "bXktcGFzc3dvcmQ=" | base64 --decode
# Output: my-password
```

> ⚠️ **Base64 BUKAN enkripsi!** Siapa saja bisa decode. Base64 hanya encoding.
> Keamanan Secret bergantung pada RBAC (siapa boleh baca) dan encryption at rest.

### Tipe-tipe Secret:

| Type | Use Case |
|------|----------|
| `Opaque` | Default, key-value bebas |
| `kubernetes.io/dockerconfigjson` | Docker registry credentials |
| `kubernetes.io/tls` | TLS certificate & key |
| `kubernetes.io/basic-auth` | Username & password |

### Membuat Secret via CLI:

```bash
# Dari literal (otomatis di-encode base64)
kubectl create secret generic db-secret \
  --from-literal=DB_PASSWORD=supersecret \
  -n taskmanager

# Dari file (misalnya TLS cert)
kubectl create secret tls my-tls \
  --cert=cert.pem \
  --key=key.pem \
  -n taskmanager
```

---

## Best Practices Keamanan

### 1. Jangan Commit Secret ke Git!

```bash
# Tambahkan ke .gitignore
echo "k8s/secret.yaml" >> .gitignore
```

### 2. Gunakan Template untuk Secret

```yaml
# k8s/secret.yaml.template (commit ini)
apiVersion: v1
kind: Secret
metadata:
  name: taskmanager-secret
  namespace: taskmanager
type: Opaque
data:
  DATABASE_URL: <BASE64_ENCODED_DATABASE_URL>
  JWT_SECRET: <BASE64_ENCODED_JWT_SECRET>
```

### 3. Cara Rotate Secret

```bash
# 1. Update Secret
kubectl create secret generic taskmanager-secret \
  --from-literal=JWT_SECRET=new-super-secret-key \
  --from-literal=DATABASE_URL='postgres://...' \
  -n taskmanager \
  --dry-run=client -o yaml | kubectl apply -f -

# 2. Restart pods agar ambil secret baru
kubectl rollout restart deployment/backend -n taskmanager

# 3. Verifikasi
kubectl get pods -n taskmanager -l component=backend
```

### 4. Di Production: Gunakan External Secret Manager

```
Google Secret Manager → External Secrets Operator → K8s Secret → Pod

Keuntungan:
- Secret di-enkripsi sepenuhnya
- Audit trail (siapa akses kapan)
- Auto-rotation
- Tidak perlu manage base64
```

---

## Eksperimen: Update Config Tanpa Rebuild

```bash
# 1. Ubah log level dari info ke debug
kubectl edit configmap taskmanager-config -n taskmanager
# Ubah RUST_LOG menjadi "taskmanager_api=debug,tower_http=debug"

# 2. Restart pods (ConfigMap tidak auto-refresh di env var)
kubectl rollout restart deployment/backend -n taskmanager

# 3. Cek logs — sekarang lebih verbose!
kubectl logs -l component=backend -n taskmanager --tail=20
```

---

## Checklist ✅

- [ ] Paham perbedaan ConfigMap vs Secret
- [ ] Paham 3 cara inject ConfigMap (envFrom, valueFrom, volume)
- [ ] Bisa encode/decode base64
- [ ] Paham bahwa base64 ≠ enkripsi
- [ ] Sudah coba update ConfigMap dan restart pod
- [ ] Paham best practices keamanan Secret

---

**Selanjutnya**: [Fase 6 — Database di Kubernetes →](./06-database.md)
