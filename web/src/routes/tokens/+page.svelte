<script lang="ts">
  import { onMount } from 'svelte';
  import { tokensApi, projectsApi, type Token, type TokenCreated, type Project } from '$lib/api';

  let tokens: Token[] = [];
  let projects: Project[] = [];
  let loading = true;
  let error = '';
  let showForm = false;
  let newName = '';
  let selectedProjectIds: string[] = [];
  let creating = false;
  let justCreated: TokenCreated | null = null;
  let copied = false;

  onMount(async () => {
    try {
      [tokens, projects] = await Promise.all([tokensApi.list(), projectsApi.list()]);
    } catch (e: any) {
      error = e.message;
    } finally {
      loading = false;
    }
  });

  async function createToken() {
    if (!newName.trim()) return;
    creating = true;
    try {
      const t = await tokensApi.create(newName.trim(), selectedProjectIds);
      tokens = [t, ...tokens];
      justCreated = t;
      newName = ''; selectedProjectIds = []; showForm = false;
    } catch (e: any) {
      error = e.message;
    } finally {
      creating = false;
    }
  }

  async function deleteToken(t: Token) {
    if (!confirm(`Revoke token "${t.name}"?`)) return;
    try {
      await tokensApi.delete(t.id);
      tokens = tokens.filter(x => x.id !== t.id);
      if (justCreated?.id === t.id) justCreated = null;
    } catch (e: any) {
      error = e.message;
    }
  }

  function toggleProject(id: string) {
    if (selectedProjectIds.includes(id)) {
      selectedProjectIds = selectedProjectIds.filter(x => x !== id);
    } else {
      selectedProjectIds = [...selectedProjectIds, id];
    }
  }

  async function copy(text: string) {
    await navigator.clipboard.writeText(text);
    copied = true;
    setTimeout(() => copied = false, 1500);
  }

  function formatDate(s: string | null) {
    if (!s) return 'Never';
    return new Date(s + 'Z').toLocaleString();
  }
</script>

<div class="page-header">
  <h2>API Tokens</h2>
  <button class="btn-primary" on:click={() => showForm = !showForm}>
    {showForm ? 'Cancel' : '+ New token'}
  </button>
</div>

{#if error}
  <div class="error-msg" style="margin-bottom:16px">{error}</div>
{/if}

{#if justCreated}
  <div class="card token-reveal">
    <div class="reveal-header">
      <span class="success-icon">✓</span>
      <strong>Token created — copy it now, it won't be shown again</strong>
    </div>
    <div class="token-value">
      <code>{justCreated.token}</code>
      <button class="btn-ghost copy-btn" on:click={() => copy(justCreated!.token)}>
        {copied ? 'Copied!' : 'Copy'}
      </button>
    </div>
    <button class="btn-ghost dismiss" on:click={() => justCreated = null}>Dismiss</button>
  </div>
{/if}

{#if showForm}
  <div class="card new-form">
    <h3>New token</h3>
    <form on:submit|preventDefault={createToken}>
      <label>
        Name
        <input type="text" bind:value={newName} placeholder="macbook-dev" required />
      </label>

      {#if projects.length > 0}
        <div class="scope-section">
          <div class="scope-label">Scope to projects <span class="muted">(leave empty for all projects)</span></div>
          <div class="project-checks">
            {#each projects as p}
              <label class="check-label">
                <input
                  type="checkbox"
                  checked={selectedProjectIds.includes(p.id)}
                  on:change={() => toggleProject(p.id)}
                  style="width:auto"
                />
                {p.name} <span class="muted">({p.slug})</span>
              </label>
            {/each}
          </div>
        </div>
      {/if}

      <div class="form-actions">
        <button type="submit" class="btn-primary" disabled={creating || !newName.trim()}>
          {creating ? 'Creating…' : 'Create token'}
        </button>
      </div>
    </form>
  </div>
{/if}

{#if loading}
  <div class="empty">Loading…</div>
{:else if tokens.length === 0}
  <div class="card empty">No tokens yet.</div>
{:else}
  <div class="card table-card">
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Scope</th>
          <th>Last used</th>
          <th>Created</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each tokens as t}
          <tr>
            <td><strong>{t.name}</strong></td>
            <td>
              {#if t.project_ids.length === 0}
                <span class="badge all">All projects</span>
              {:else}
                {#each t.project_ids as pid}
                  <span class="badge">{projects.find(p => p.id === pid)?.slug ?? pid}</span>
                {/each}
              {/if}
            </td>
            <td class="muted-cell">{formatDate(t.last_used_at)}</td>
            <td class="muted-cell">{formatDate(t.created_at)}</td>
            <td>
              <button class="btn-danger icon-btn" on:click={() => deleteToken(t)} title="Revoke">Revoke</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .page-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
  }

  h2 { font-size: 20px; font-weight: 700; }

  .token-reveal {
    border-color: var(--success);
    margin-bottom: 20px;
  }

  .reveal-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    margin-bottom: 12px;
  }

  .success-icon { color: var(--success); font-size: 16px; }

  .token-value {
    display: flex;
    align-items: center;
    gap: 10px;
    background: var(--bg3);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 10px 14px;
  }

  .token-value code {
    flex: 1;
    font-family: monospace;
    font-size: 13px;
    word-break: break-all;
  }

  .copy-btn { flex-shrink: 0; }
  .dismiss { margin-top: 12px; font-size: 12px; }

  .new-form { margin-bottom: 24px; }
  .new-form h3 { font-size: 15px; font-weight: 600; margin-bottom: 16px; }

  form { display: flex; flex-direction: column; gap: 14px; }

  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .scope-section { display: flex; flex-direction: column; gap: 8px; }
  .scope-label { font-size: 12px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.05em; }
  .muted { color: var(--muted); font-weight: 400; text-transform: none; letter-spacing: 0; }

  .project-checks { display: flex; flex-wrap: wrap; gap: 8px; }

  .check-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--text);
    background: var(--bg3);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 5px 10px;
    cursor: pointer;
    text-transform: none;
    letter-spacing: 0;
  }

  .form-actions { display: flex; justify-content: flex-end; }

  .table-card { padding: 0; overflow: hidden; }
  table { width: 100%; border-collapse: collapse; }
  th {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
    padding: 10px 16px;
    text-align: left;
    border-bottom: 1px solid var(--border);
  }
  td { padding: 12px 16px; border-bottom: 1px solid var(--border); font-size: 13px; }
  tr:last-child td { border-bottom: none; }

  .badge {
    background: var(--bg3);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 11px;
    padding: 2px 8px;
    margin-right: 4px;
    font-family: monospace;
  }
  .badge.all { color: var(--muted); }
  .muted-cell { color: var(--muted); font-size: 12px; }
  .icon-btn { padding: 5px 10px; font-size: 12px; }

  .empty { color: var(--muted); padding: 40px; text-align: center; }
</style>
