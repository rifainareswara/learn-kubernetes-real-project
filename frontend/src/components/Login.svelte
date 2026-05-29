<script>
  import { login, register } from '../lib/api.js';

  let { onLogin, showToast } = $props();

  let mode = $state('login');
  let email = $state('');
  let password = $state('');
  let username = $state('');
  let loading = $state(false);
  let error = $state('');

  async function handleSubmit(e) {
    e.preventDefault();
    loading = true;
    error = '';

    try {
      if (mode === 'register') {
        await register(username, email, password);
        showToast('Registrasi berhasil! Silakan login.', 'success');
        mode = 'login';
        password = '';
      } else {
        const data = await login(email, password);
        onLogin(data.user || { username: email.split('@')[0] });
      }
    } catch (err) {
      error = err.message;
      showToast(err.message, 'error');
    } finally {
      loading = false;
    }
  }
</script>

<div class="login-container animate-fade-in">
  <div class="login-card card">
    <!-- Header -->
    <div class="login-header">
      <div class="login-icon">☸️</div>
      <h2>{mode === 'login' ? 'Welcome Back' : 'Create Account'}</h2>
      <p>{mode === 'login' ? 'Masuk ke Task Manager' : 'Daftar akun baru'}</p>
    </div>

    <!-- Form -->
    <form onsubmit={handleSubmit} class="login-form">
      {#if mode === 'register'}
        <div class="input-group animate-slide-in">
          <label for="username">Username</label>
          <input
            id="username"
            class="input"
            type="text"
            placeholder="johndoe"
            bind:value={username}
            required
            minlength="3"
          />
        </div>
      {/if}

      <div class="input-group">
        <label for="email">Email</label>
        <input
          id="email"
          class="input"
          type="email"
          placeholder="john@example.com"
          bind:value={email}
          required
        />
      </div>

      <div class="input-group">
        <label for="password">Password</label>
        <input
          id="password"
          class="input"
          type="password"
          placeholder="••••••••"
          bind:value={password}
          required
          minlength="6"
        />
      </div>

      {#if error}
        <div class="error-message">{error}</div>
      {/if}

      <button class="btn btn-primary login-btn" type="submit" disabled={loading}>
        {#if loading}
          <span class="spinner"></span>
        {:else}
          {mode === 'login' ? '🚀 Login' : '✨ Daftar'}
        {/if}
      </button>
    </form>

    <!-- Toggle Mode -->
    <div class="login-toggle">
      {#if mode === 'login'}
        <p>Belum punya akun?
          <button class="link-btn" onclick={() => { mode = 'register'; error = ''; }}>Daftar di sini</button>
        </p>
      {:else}
        <p>Sudah punya akun?
          <button class="link-btn" onclick={() => { mode = 'login'; error = ''; }}>Login di sini</button>
        </p>
      {/if}
    </div>
  </div>
</div>

<style>
  .login-container {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 60vh;
  }

  .login-card {
    width: 100%;
    max-width: 420px;
    padding: 40px;
  }

  .login-header {
    text-align: center;
    margin-bottom: 32px;
  }

  .login-icon {
    font-size: 3rem;
    margin-bottom: 12px;
  }

  .login-header h2 {
    font-size: 1.5rem;
    font-weight: 700;
    margin-bottom: 4px;
  }

  .login-header p {
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .login-form {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .login-btn {
    width: 100%;
    padding: 12px;
    font-size: 1rem;
    margin-top: 8px;
  }

  .error-message {
    padding: 10px 14px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.2);
    border-radius: var(--radius-md);
    color: #f87171;
    font-size: 0.85rem;
  }

  .login-toggle {
    text-align: center;
    margin-top: 24px;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  .link-btn {
    background: none;
    border: none;
    color: var(--accent-primary);
    cursor: pointer;
    font-family: inherit;
    font-size: inherit;
    font-weight: 500;
    transition: color var(--transition-fast);
  }

  .link-btn:hover {
    color: var(--accent-secondary);
  }
</style>
