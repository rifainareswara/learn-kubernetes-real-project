# ☸️ Learn Kubernetes — Real Project

Belajar Kubernetes dari nol menggunakan **real fullstack project** yang di-deploy ke **Google Kubernetes Engine (GKE)**.

## 🏗️ Tech Stack

| Layer | Technology |
|-------|-----------|
| **Frontend** | Svelte 5 + Vite + Nginx |
| **Backend** | Rust Axum + SQLx + JWT |
| **Database** | PostgreSQL 16 |
| **Container** | Docker (multi-stage build) |
| **Orchestration** | Kubernetes (GKE Autopilot) |
| **Registry** | Google Artifact Registry |
| **CI/CD** | GitHub Actions |

## 🏛️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Google Kubernetes Engine                   │
│                                                               │
│  ┌──────────┐    ┌──────────┐    ┌─────────────────────┐    │
│  │ Svelte   │    │ Rust     │    │ PostgreSQL          │    │
│  │ Frontend │───▶│ Backend  │───▶│ (StatefulSet +      │    │
│  │ (Nginx)  │    │ API      │    │  PersistentVolume)  │    │
│  └────┬─────┘    └────┬─────┘    └─────────────────────┘    │
│       │               │                                      │
│  ┌────┴─────┐    ┌────┴─────┐                               │
│  │ Service  │    │ Service  │                               │
│  └────┬─────┘    └────┴─────┘                               │
│       │               │                                      │
│  ┌────┴───────────────┴───────┐                             │
│  │      Ingress Controller     │                             │
│  └────────────┬───────────────┘                             │
└───────────────┼─────────────────────────────────────────────┘
                │
           ☁️ Internet
```

## 📚 Tutorial (Bahasa Indonesia)

Tutorial step-by-step dari nol, tersedia di folder `docs/`:

| Fase | Topik | File |
|------|-------|------|
| 0 | Prerequisites & Setup GCP | [docs/00-prerequisites.md](docs/00-prerequisites.md) |
| 1 | Konsep Dasar Kubernetes | [docs/01-konsep-dasar.md](docs/01-konsep-dasar.md) |
| 2 | Containerization (Docker) | [docs/02-containerization.md](docs/02-containerization.md) |
| 3 | Deploy Pertama ke K8s 🚀 | [docs/03-first-deployment.md](docs/03-first-deployment.md) |
| 4 | Config & Secrets | [docs/04-config-secrets.md](docs/04-config-secrets.md) |
| 5 | Database (StatefulSet) | [docs/05-database.md](docs/05-database.md) |
| 6 | Networking (Ingress) | [docs/06-networking.md](docs/06-networking.md) |
| 7 | Observability | [docs/07-observability.md](docs/07-observability.md) |
| 8 | Production Best Practices | [docs/08-production.md](docs/08-production.md) |

## 🚀 Quick Start

### Prerequisites

- Docker Desktop
- kubectl
- gcloud CLI
- GCP account (trial credit OK!)

### Local Development (Docker Compose)

```bash
# Clone project
cd learn-kubernetes-real-project

# Jalankan seluruh stack
docker compose up --build

# Akses:
# Frontend: http://localhost:8080
# Backend:  http://localhost:3000
# Swagger:  http://localhost:3000/swagger-ui/
```

### Deploy ke GKE

```bash
# 1. Setup GCP (lihat docs/00-prerequisites.md)
gcloud container clusters create-auto taskmanager-cluster --region=asia-southeast2

# 2. Build & push images
export REGISTRY=asia-southeast2-docker.pkg.dev/YOUR_PROJECT/taskmanager
docker build -t $REGISTRY/backend:v1 ./backend && docker push $REGISTRY/backend:v1
docker build -t $REGISTRY/frontend:v1 ./frontend && docker push $REGISTRY/frontend:v1

# 3. Update image paths di k8s/*.yaml

# 4. Deploy!
kubectl apply -f k8s/

# 5. Dapatkan IP
kubectl get ingress -n taskmanager
```

## 📁 Project Structure

```
learn-kubernetes-real-project/
├── README.md
├── docker-compose.yml              # Local development
├── backend/                        # Rust Axum API
│   ├── Cargo.toml
│   ├── Dockerfile                  # Multi-stage build (~80MB)
│   ├── migrations/
│   └── src/
├── frontend/                       # Svelte 5 + Vite
│   ├── package.json
│   ├── Dockerfile                  # Multi-stage build (~25MB)
│   ├── nginx.conf
│   └── src/
├── k8s/                            # Kubernetes manifests
│   ├── namespace.yaml
│   ├── configmap.yaml
│   ├── secret.yaml
│   ├── postgres-statefulset.yaml
│   ├── postgres-service.yaml
│   ├── backend-deployment.yaml
│   ├── backend-service.yaml
│   ├── frontend-deployment.yaml
│   ├── frontend-service.yaml
│   ├── ingress.yaml
│   ├── hpa.yaml
│   └── network-policy.yaml
├── .github/workflows/
│   └── deploy.yml                  # CI/CD pipeline
└── docs/                           # Tutorial (Bahasa Indonesia)
    ├── 00-prerequisites.md
    ├── 01-konsep-dasar.md
    ├── 02-containerization.md
    ├── 03-first-deployment.md
    ├── 04-config-secrets.md
    ├── 05-database.md
    ├── 06-networking.md
    ├── 07-observability.md
    └── 08-production.md
```

## 📋 API Endpoints

| Method | Endpoint | Auth | Description |
|--------|----------|:----:|------------|
| `GET` | `/health` | ❌ | Health check |
| `POST` | `/api/auth/register` | ❌ | Register |
| `POST` | `/api/auth/login` | ❌ | Login → JWT |
| `GET` | `/api/auth/me` | ✅ | Profile |
| `POST` | `/api/tasks` | ✅ | Create task |
| `GET` | `/api/tasks` | ✅ | List tasks |
| `GET` | `/api/tasks/{id}` | ✅ | Get task |
| `PUT` | `/api/tasks/{id}` | ✅ | Update task |
| `DELETE` | `/api/tasks/{id}` | ✅ | Delete task |

## 💡 Kubernetes Concepts Covered

- Pod, Deployment, StatefulSet
- Service (ClusterIP, Headless)
- ConfigMap & Secret
- PersistentVolume & PersistentVolumeClaim
- Ingress & Load Balancer
- Health Probes (Liveness, Readiness, Startup)
- Resource Requests & Limits
- HPA (Horizontal Pod Autoscaler)
- Network Policy
- Rolling Update & Rollback
- CI/CD with GitHub Actions

## ⚠️ Cost Management

Estimated cost: ~$1.76/day (GKE Autopilot).

```bash
# ALWAYS delete cluster when not learning!
gcloud container clusters delete taskmanager-cluster --region=asia-southeast2 --quiet
```
