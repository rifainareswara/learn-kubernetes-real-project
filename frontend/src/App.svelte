<script>
  import { isLoggedIn, logout, getMe } from './lib/api.js';
  import Login from './components/Login.svelte';
  import TaskList from './components/TaskList.svelte';

  let page = $state(isLoggedIn() ? 'tasks' : 'login');
  let user = $state(null);
  let toast = $state(null);

  // Check auth on mount
  if (isLoggedIn()) {
    getMe()
      .then((data) => (user = data))
      .catch(() => {
        logout();
        page = 'login';
      });
  }

  function handleLogin(userData) {
    user = userData;
    page = 'tasks';
    showToast('Login berhasil! 🎉', 'success');
  }

  function handleLogout() {
    logout();
    user = null;
    page = 'login';
    showToast('Logout berhasil', 'success');
  }

  function showToast(message, type = 'success') {
    toast = { message, type };
    setTimeout(() => (toast = null), 3000);
  }
</script>

<div class="app-layout">
  <!-- Header -->
  <header class="app-header">
    <div class="header-content">
      <div class="logo">
        <span class="logo-icon">☸️</span>
        <h1>Task Manager</h1>
        <span class="logo-badge">K8s Project</span>
      </div>
      {#if user}
        <div class="user-section">
          <span class="user-greeting">Halo, <strong>{user.username}</strong></span>
          <button class="btn btn-secondary btn-sm" onclick={handleLogout}>Logout</button>
        </div>
      {/if}
    </div>
  </header>

  <!-- Main Content -->
  <main class="app-main">
    {#if page === 'login'}
      <Login onLogin={handleLogin} {showToast} />
    {:else if page === 'tasks'}
      <TaskList {showToast} />
    {/if}
  </main>

  <!-- Footer -->
  <footer class="app-footer">
    <p>🦀 Rust Axum + ⚡ Svelte + ☸️ Kubernetes — Learning Project</p>
  </footer>
</div>

<!-- Toast -->
{#if toast}
  <div class="toast toast-{toast.type}">
    {toast.message}
  </div>
{/if}

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .app-header {
    background: var(--bg-glass);
    border-bottom: 1px solid var(--border-default);
    backdrop-filter: blur(20px);
    position: sticky;
    top: 0;
    z-index: 100;
  }

  .header-content {
    max-width: 1200px;
    margin: 0 auto;
    padding: 16px 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .logo-icon {
    font-size: 1.5rem;
  }

  .logo h1 {
    font-size: 1.25rem;
    font-weight: 700;
    background: var(--accent-gradient);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }

  .logo-badge {
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    padding: 3px 8px;
    border-radius: var(--radius-full);
    background: rgba(129, 140, 248, 0.15);
    color: var(--accent-primary);
    border: 1px solid rgba(129, 140, 248, 0.2);
  }

  .user-section {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .user-greeting {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  .app-main {
    flex: 1;
    max-width: 1200px;
    margin: 0 auto;
    padding: 32px 24px;
    width: 100%;
  }

  .app-footer {
    text-align: center;
    padding: 24px;
    color: var(--text-muted);
    font-size: 0.8rem;
    border-top: 1px solid var(--border-default);
  }
</style>
