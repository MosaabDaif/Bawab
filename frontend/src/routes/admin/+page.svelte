<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { login, getToken } from '$lib/api';

  let username = '';
  let password = '';
  let errorMsg = '';
  let loading = false;

  onMount(() => {
    if (getToken()) goto('/dashboard');
  });

  async function handleLogin() {
    errorMsg = '';
    if (!username || !password) {
      errorMsg = 'Please fill in both fields.';
      return;
    }
    loading = true;
    try {
      const { token } = await login(username, password);
      localStorage.setItem('jwt', token);
      goto('/dashboard');
    } catch (e) {
      errorMsg = (e as Error).message;
    } finally {
      loading = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleLogin();
  }
</script>

<svelte:head>
  <title>Admin Login — Library</title>
</svelte:head>

<div class="center">
  <div class="login-card card">
    <div class="logo">🔒</div>
    <h1>Admin Login</h1>
    <p class="subtitle">Library management dashboard</p>

    <div class="form-group">
      <label for="username">Username</label>
      <input
        id="username"
        type="text"
        placeholder="admin"
        autocomplete="username"
        bind:value={username}
        on:keydown={onKeydown}
        disabled={loading}
      />
    </div>

    <div class="form-group">
      <label for="password">Password</label>
      <input
        id="password"
        type="password"
        placeholder="••••••••"
        autocomplete="current-password"
        bind:value={password}
        on:keydown={onKeydown}
        disabled={loading}
      />
    </div>

    {#if errorMsg}
      <p class="error-msg">{errorMsg}</p>
    {/if}

    <button
      class="btn btn-primary btn-block"
      style="margin-top: 0.5rem;"
      on:click={handleLogin}
      disabled={loading}
    >
      {loading ? 'Signing in…' : 'Sign In'}
    </button>

    <a class="back-link" href="/">← Back to check-in</a>
  </div>
</div>

<style>
  .login-card {
    width: 100%;
    max-width: 380px;
    text-align: center;
    padding: 2.5rem 2rem;
  }

  .logo {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 700;
    margin-bottom: 0.25rem;
  }

  .subtitle {
    color: var(--text-muted);
    font-size: 0.9rem;
    margin-bottom: 1.5rem;
  }

  /* Override label to left-align inside the card */
  label { text-align: left; }

  .back-link {
    display: block;
    margin-top: 1.25rem;
    font-size: 0.85rem;
    color: var(--text-muted);
    text-decoration: none;
  }
  .back-link:hover { color: var(--primary); }
</style>
