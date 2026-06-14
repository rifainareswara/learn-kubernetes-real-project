# 🖥️ Menjalankan di Lokal (minikube)

Overlay Kustomize untuk menjalankan project ini di **minikube**, tanpa GCP & tanpa biaya.
Folder `k8s/` asli **tidak diubah** (tetap untuk deploy ke GKE).

## Yang di-override overlay ini

| Hal | GKE (`k8s/`) | Lokal (`k8s-local/`) |
|-----|--------------|----------------------|
| Image backend/frontend | Artifact Registry | `taskmanager/*:local` (build lokal) |
| `imagePullPolicy` | `Always` | `IfNotPresent` |
| StorageClass postgres | `standard-rwo` | `standard` (default minikube) |
| Ingress class | `gce` | `nginx` + host `taskmanager.local` |

## Langkah

```bash
# 1. Start cluster + addon
minikube start --cpus=4 --memory=6144 --driver=docker
minikube addons enable ingress          # Ingress nginx
minikube addons enable metrics-server   # untuk HPA (k8s/12-hpa.yaml)

# 2. Build image LANGSUNG di dalam Docker minikube (tanpa registry/push)
eval $(minikube docker-env)
docker build -t taskmanager/backend:local  ./backend
docker build -t taskmanager/frontend:local ./frontend
eval $(minikube docker-env -u)           # kembalikan docker ke laptop

# 3. Deploy overlay lokal
#    (-k tidak support --load-restrictor, jadi pakai pipe)
kubectl kustomize k8s-local --load-restrictor=LoadRestrictionsNone | kubectl apply -f -

# 4. Tunggu semua pod Running
kubectl get pods -n taskmanager -w

# 5. Akses lewat Ingress
echo "$(minikube ip) taskmanager.local" | sudo tee -a /etc/hosts
# buka http://taskmanager.local
```

> 💡 Preview build tanpa apply: `kubectl kustomize k8s-local --load-restrictor=LoadRestrictionsNone`

## Akses alternatif (tanpa Ingress)

```bash
kubectl port-forward -n taskmanager svc/frontend-service 8080:80
kubectl port-forward -n taskmanager svc/backend-service  3000:3000
```

## Bersih-bersih

```bash
kubectl delete -k k8s-local --load-restrictor=LoadRestrictionsNone 2>/dev/null \
  || kubectl kustomize k8s-local --load-restrictor=LoadRestrictionsNone | kubectl delete -f -
minikube stop          # matikan cluster (tetap tersimpan)
# minikube delete      # hapus total
```

## Troubleshooting

- **`ImagePullBackOff`** → image belum di-build di Docker minikube. Ulangi step 2 (pastikan `eval $(minikube docker-env)` sebelum `docker build`).
- **Pod backend `CrashLoopBackOff`** → cek log: `kubectl logs -n taskmanager -l component=backend`. Biasanya gagal connect DB / migration; pastikan pod `postgres-0` sudah `Running`.
- **HPA `<unknown>/70%`** → metrics-server belum siap. Tunggu ~1 menit atau `minikube addons enable metrics-server`.
- **Ingress tidak jalan** → pastikan `minikube addons enable ingress` & entry `/etc/hosts` sudah benar (`minikube ip`).
