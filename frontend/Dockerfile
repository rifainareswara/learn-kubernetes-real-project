# =============================================================================
# Dockerfile — Svelte Frontend (Multi-Stage Build)
# =============================================================================
#
# Stage 1 (builder): npm install + vite build
# Stage 2 (runtime): Copy static files ke Nginx
#
# Hasil: image ~25MB yang serve static files via Nginx
#
# Build:  docker build -t taskmanager-frontend .
# Run:    docker run -p 8080:80 taskmanager-frontend
# =============================================================================

# ── Stage 1: Build ───────────────────────────────────────────────────────────
FROM node:lts-alpine AS builder

WORKDIR /app

# Copy package files dulu (untuk cache npm install)
COPY package.json package-lock.json* ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build untuk production
# VITE_API_URL dikosongkan karena di K8s, Nginx akan proxy ke backend
RUN VITE_API_URL="" npm run build

# ── Stage 2: Serve dengan Nginx ──────────────────────────────────────────────
FROM nginx:alpine AS runtime

# Copy custom nginx config sebagai TEMPLATE.
# Entrypoint nginx (20-envsubst-on-templates.sh) menjalankan envsubst pada
# semua file di /etc/nginx/templates/*.template, mengganti ${BACKEND_HOST}
# dengan nilai env var, lalu menulis hasilnya ke /etc/nginx/conf.d/default.conf.
# Penting: envsubst hanya mengganti variabel yang ADA di environment, jadi
# variabel nginx sendiri ($uri, $host, $remote_addr, dst) tetap aman.
COPY nginx.conf /etc/nginx/templates/default.conf.template

# Copy build output dari stage 1
COPY --from=builder /app/dist /usr/share/nginx/html

# Expose port 80
EXPOSE 80

# Nginx berjalan di foreground (required untuk Docker)
CMD ["nginx", "-g", "daemon off;"]
