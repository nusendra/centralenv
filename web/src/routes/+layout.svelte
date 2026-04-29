<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { authed, setAuthed } from '$lib/auth';
  import { authApi } from '$lib/api';

  onMount(async () => {
    try {
      await authApi.me();
      setAuthed(true);
    } catch {
      setAuthed(false);
      if ($page.url.pathname !== '/login') goto('/login');
    }
  });

  async function logout() {
    await authApi.logout().catch(() => {});
    setAuthed(false);
    goto('/login');
  }
</script>

{#if $authed === false && $page.url.pathname !== '/login'}
  <div class="loading">Redirecting...</div>
{:else}
  {#if $page.url.pathname !== '/login'}
    <nav>
      <div class="nav-brand">
        <span class="brand-icon">⚡</span>
        <span class="brand-name">CentralEnv</span>
      </div>
      <div class="nav-links">
        <a href="/projects" class:active={$page.url.pathname.startsWith('/projects')}>Projects</a>
        <a href="/tokens" class:active={$page.url.pathname.startsWith('/tokens')}>Tokens</a>
      </div>
      <button class="btn-ghost" on:click={logout}>Logout</button>
    </nav>
  {/if}
  <main>
    <slot />
  </main>
{/if}

<style>
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    color: var(--muted);
  }

  nav {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 0 24px;
    height: 56px;
    background: var(--bg2);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .nav-brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    font-size: 15px;
    margin-right: 8px;
  }

  .brand-icon { font-size: 18px; }

  .nav-links {
    display: flex;
    gap: 4px;
    flex: 1;
  }

  .nav-links a {
    padding: 5px 12px;
    border-radius: 6px;
    color: var(--muted);
    font-size: 13px;
    font-weight: 500;
  }
  .nav-links a:hover, .nav-links a.active {
    background: var(--bg3);
    color: var(--text);
  }

  main {
    max-width: 960px;
    margin: 0 auto;
    padding: 32px 24px;
  }
</style>
