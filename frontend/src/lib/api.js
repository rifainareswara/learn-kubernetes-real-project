// =============================================================================
// API Helper — Centralized fetch wrapper
// =============================================================================

const API_BASE = import.meta.env.VITE_API_URL || '';

function getToken() {
  return localStorage.getItem('token');
}

async function request(method, path, body = null) {
  const headers = { 'Content-Type': 'application/json' };
  const token = getToken();
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const options = { method, headers };
  if (body) {
    options.body = JSON.stringify(body);
  }

  const res = await fetch(`${API_BASE}${path}`, options);
  const data = await res.json().catch(() => null);

  if (!res.ok) {
    const message = data?.error?.message || data?.message || `Error ${res.status}`;
    throw new Error(message);
  }

  return data;
}

// ── Auth API ─────────────────────────────────────────────────────────────────
export async function register(username, email, password) {
  return request('POST', '/api/auth/register', { username, email, password });
}

export async function login(email, password) {
  const data = await request('POST', '/api/auth/login', { email, password });
  if (data.token) {
    localStorage.setItem('token', data.token);
  }
  return data;
}

export async function getMe() {
  return request('GET', '/api/auth/me');
}

export function logout() {
  localStorage.removeItem('token');
}

export function isLoggedIn() {
  return !!getToken();
}

// ── Tasks API ────────────────────────────────────────────────────────────────
export async function getTasks(params = {}) {
  const query = new URLSearchParams();
  if (params.status) query.set('status', params.status);
  if (params.page) query.set('page', params.page);
  if (params.per_page) query.set('per_page', params.per_page);

  const qs = query.toString();
  return request('GET', `/api/tasks${qs ? '?' + qs : ''}`);
}

export async function getTask(id) {
  return request('GET', `/api/tasks/${id}`);
}

export async function createTask(data) {
  return request('POST', '/api/tasks', data);
}

export async function updateTask(id, data) {
  return request('PUT', `/api/tasks/${id}`, data);
}

export async function deleteTask(id) {
  return request('DELETE', `/api/tasks/${id}`);
}

// ── Health Check ─────────────────────────────────────────────────────────────
export async function healthCheck() {
  return request('GET', '/health');
}
