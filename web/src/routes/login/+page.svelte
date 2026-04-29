<script lang="ts">
  import { goto } from '$app/navigation';
  import { authApi } from '$lib/api';
  import { setAuthed } from '$lib/auth';

  let username = '';
  let password = '';
  let error = '';
  let loading = false;

  async function submit() {
    error = '';
    loading = true;
    try {
      await authApi.login(username, password);
      setAuthed(true);
      goto('/projects');
    } catch (e: any) {
      error = e.message === 'UNAUTHORIZED' ? 'Invalid username or password' : e.message;
    } finally {
      loading = false;
    }
  }
</script>

<div class="wrap">
  <div class="card login-card">
    <div class="header">
      <span class="icon">⚡</span>
      <h1>CentralEnv</h1>
      <p>Sign in to manage your environment variables</p>
    </div>

    {#if error}
      <div class="error-msg">{error}</div>
    {/if}

    <form on:submit|preventDefault={submit}>
      <label>
        Username
        <input type="text" bind:value={username} autocomplete="username" required />
      </label>
      <label>
        Password
        <input type="password" bind:value={password} autocomplete="current-password" required />
      </label>
      <button type="submit" class="btn-primary" disabled={loading}>
        {loading ? 'Signing in…' : 'Sign in'}
      </button>
    </form>
  </div>
</div>

<style>
  .wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 24px;
  }

  .login-card {
    width: 100%;
    max-width: 380px;
  }

  .header {
    text-align: center;
    margin-bottom: 24px;
  }

  .icon { font-size: 36px; }

  h1 {
    font-size: 22px;
    font-weight: 700;
    margin: 8px 0 6px;
  }

  .header p { color: var(--muted); font-size: 13px; }

  form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    font-weight: 500;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  button[type=submit] {
    width: 100%;
    padding: 10px;
    font-size: 14px;
    margin-top: 4px;
  }

  .error-msg { margin-bottom: 16px; }
</style>
