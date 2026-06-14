# Fase 3: Run Lokal di Minikube 🖥️

> **Tujuan**: Menjalankan seluruh stack (Postgres + Backend + Frontend) di
> Kubernetes **lokal** pakai minikube — gratis, tanpa GCP, dan cepat untuk iterasi.

## Kenapa Lokal Dulu?

Sebelum keluar biaya & kuota GKE (Fase 4), kamu bisa menjalankan cluster
Kubernetes asli di laptop dengan **minikube**. Manfaatnya:

- 💸 **Gratis** — tidak menyentuh trial credit GCP.
- ⚡ **Cepat** — build image langsung di Docker minikube, tanpa push ke registry.
- 🔁 **Aman bereksperimen** — rusak? `minikube delete` lalu mulai lagi.

Yang penting: **folder `k8s/` asli tidak diubah**. Kita pakai _overlay Kustomize_
di `k8s-local/` yang menimpa beberapa field saja, supaya manifest yang sama tetap
bisa dipakai untuk GKE nanti.

## Apa yang Akan Kamu Pelajari
- Konsep **Kustomize base + overlay** (satu manifest, banyak environment)
- Build image langsung ke dalam Docker daemon minikube (tanpa registry)
- Perbedaan konfigurasi lokal vs GKE (image, StorageClass, Ingress)
- Akses aplikasi via Ingress lokal & port-forward

---

## Konsep: Base + Overlay

Daripada menyalin-tempel manifest untuk tiap environment, Kustomize memakai
satu **base** (folder `k8s/`) lalu **overlay** kecil yang hanya berisi
perbedaannya.

```
k8s/            ← BASE: manifest lengkap (dipakai apa adanya di GKE)
   └── 01..12 yaml

k8s-local/      ← OVERLAY: hanya bedanya untuk minikube
   ├── kustomization.yaml          ← daftar base + patch
   ├── patch-image-pull-policy.yaml
   ├── patch-ingress.yaml
   └── patch-postgres-storageclass.yaml
```

Yang ditimpa overlay `k8s-local/`:

| Hal | GKE (`k8s/`) | Lokal (`k8s-local/`) | Kenapa |
|-----|--------------|----------------------|--------|
| Image backend/frontend | Artifact Registry | `taskmanager-backend:v1`, `taskmanager-frontend:v1` | image dibuat lokal, tak ada di registry |
| `imagePullPolicy` | `Always` | `IfNotPresent` | jangan coba pull dari registry — pakai image di node |
| StorageClass Postgres | `standard-rwo` | `standard` | minikube tak punya Google Persistent Disk |
| Ingress class | `gce` | `nginx` + host `taskmanager.local` | minikube pakai ingress-nginx addon |

---

## Persiapan

Pastikan sudah terpasang (lihat juga [Fase 0](./00-prerequisites.md)):
1. ✅ Docker (driver minikube)
2. ✅ `kubectl`
3. ✅ `minikube`

```bash
minikube version
kubectl version --client
```

---

## Step 1: Start Cluster + Addon

```bash
# Cluster minikube dengan resource cukup untuk 5 pod
minikube start --cpus=4 --memory=6144 --driver=docker

# Ingress controller (untuk k8s/11-ingress.yaml)
minikube addons enable ingress

# metrics-server (dibutuhkan HPA di k8s/12-hpa.yaml)
minikube addons enable metrics-server

# Verifikasi node siap
kubectl get nodes
```

---

## Step 2: Build Image di Dalam Docker Minikube

Trik kuncinya: arahkan Docker CLI ke daemon **di dalam** minikube, lalu build di
sana. Image langsung tersedia di node tanpa perlu push ke registry.

```bash
# Arahkan docker ke daemon minikube
eval $(minikube docker-env)

# Build image — TAG harus sama persis dengan yang di kustomization.yaml
docker build -t taskmanager-backend:v1  ./backend
docker build -t taskmanager-frontend:v1 ./frontend

# (opsional) kembalikan docker ke laptop
eval $(minikube docker-env -u)
```

> ⚠️ **Tag harus cocok.** `k8s-local/kustomization.yaml` me-remap image GKE ke
> `taskmanager-backend:v1` & `taskmanager-frontend:v1`. Kalau tag beda, pod akan
> `ImagePullBackOff`.

---

## Step 3: Deploy Overlay Lokal

Base manifest ada di `../k8s` (di luar folder overlay), jadi Kustomize butuh
`--load-restrictor=LoadRestrictionsNone`. Karena `kubectl apply -k` tidak
meneruskan flag itu, kita pakai pipe:

```bash
kubectl kustomize k8s-local --load-restrictor=LoadRestrictionsNone \
  | kubectl apply -f -
```

> 💡 **Preview dulu tanpa apply** untuk melihat hasil akhir manifest:
> ```bash
> kubectl kustomize k8s-local --load-restrictor=LoadRestrictionsNone
> ```

---

## Step 4: Tunggu Semua Pod Running

```bash
kubectl get pods -n taskmanager -w
```

Yang diharapkan (tekan `Ctrl+C` saat semua `Running`):

```
NAME                        READY   STATUS    RESTARTS   AGE
backend-xxxxxxxxxx-xxxxx    1/1     Running   0          1m
backend-xxxxxxxxxx-yyyyy    1/1     Running   0          1m
frontend-xxxxxxxxxx-xxxxx   1/1     Running   0          1m
frontend-xxxxxxxxxx-yyyyy   1/1     Running   0          1m
postgres-0                  1/1     Running   0          1m
```

> 💡 Urutan startup wajar: `postgres-0` Running dulu, baru backend berhenti
> restart (backend menunggu DB siap untuk migrasi).

---

## Step 5: Akses Aplikasi

### Via Ingress (taskmanager.local)

```bash
# Petakan host lokal ke IP minikube
echo "$(minikube ip) taskmanager.local" | sudo tee -a /etc/hosts

# Buka di browser
open http://taskmanager.local        # macOS
# atau xdg-open http://taskmanager.local   # Linux
```

### Via port-forward (tanpa Ingress)

```bash
kubectl port-forward -n taskmanager svc/frontend-service 8080:80
kubectl port-forward -n taskmanager svc/backend-service  3000:3000
# Frontend: http://localhost:8080   |   Backend:  http://localhost:3000
```

---

## Update Setelah Ganti Kode

Karena tag image tetap (`:v1`) dan `imagePullPolicy: IfNotPresent`, pod **tidak**
otomatis memakai image baru. Build ulang lalu restart rollout:

```bash
eval $(minikube docker-env)
docker build -t taskmanager-backend:v1 ./backend     # atau frontend
eval $(minikube docker-env -u)

kubectl rollout restart deployment/backend -n taskmanager
kubectl rollout status  deployment/backend -n taskmanager
```

---

## Bersih-bersih

```bash
# Hapus resource (tetap simpan cluster)
kubectl kustomize k8s-local --load-restrictor=LoadRestrictionsNone \
  | kubectl delete -f -

minikube stop          # matikan cluster (data tetap tersimpan)
# minikube delete      # hapus total cluster
```

---

## Troubleshooting

- **`ImagePullBackOff`** → image belum ada di Docker minikube, atau tag salah.
  Ulangi Step 2 (pastikan `eval $(minikube docker-env)` **sebelum** `docker build`,
  dan tag = `taskmanager-backend:v1` / `taskmanager-frontend:v1`).
- **Backend `CrashLoopBackOff`** → cek `kubectl logs -n taskmanager -l component=backend`.
  Biasanya gagal connect DB / migrasi; pastikan `postgres-0` sudah `Running`.
- **Frontend `CrashLoopBackOff` (`unknown "backend_host" variable`)** → nginx config
  harus berupa _template_ (`/etc/nginx/templates/*.template`) agar `${BACKEND_HOST}`
  di-substitusi. Lihat `frontend/Dockerfile`.
- **HPA `<unknown>/70%`** → metrics-server belum siap. Tunggu ~1 menit atau
  `minikube addons enable metrics-server`.
- **Ingress tidak jalan** → pastikan `minikube addons enable ingress` & entry
  `/etc/hosts` benar (`minikube ip`).
- **Error load restriction saat apply** → selalu sertakan
  `--load-restrictor=LoadRestrictionsNone` (base ada di `../k8s`).

---

## Checklist ✅

- [ ] `minikube start` sukses & `kubectl get nodes` Ready
- [ ] Addon `ingress` & `metrics-server` enabled
- [ ] Image `taskmanager-backend:v1` & `taskmanager-frontend:v1` ter-build di Docker minikube
- [ ] `kubectl apply` overlay sukses (pakai `--load-restrictor=LoadRestrictionsNone`)
- [ ] Semua pod `Running` di namespace `taskmanager`
- [ ] Bisa akses frontend via `taskmanager.local` atau port-forward
- [ ] Paham konsep base (`k8s/`) + overlay (`k8s-local/`)

---

**Selanjutnya**: [Fase 4 — Deploy Pertama ke Kubernetes (GKE) →](./04-first-deployment.md)
