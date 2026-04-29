<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import {
    projectsApi, environmentsApi, variablesApi,
    type Project, type Environment, type Variable
  } from '$lib/api';

  const projectId = $page.params.id!;

  let project: Project | null = null;
  let environments: Environment[] = [];
  let selectedEnv: Environment | null = null;
  let loading = true;
  let varsLoading = false;
  let error = '';

  // New env form
  let newEnvName = '';
  let addingEnv = false;

  // Table editor state
  let editingVars: { key: string; value: string; masked: boolean; dirty: boolean }[] = [];
  let newKey = '';
  let newValue = '';
  let saving = false;

  // Raw editor state
  let rawMode = false;
  let rawText = '';
  let rawDirty = false;
  let rawSaving = false;
  let rawError = '';

  onMount(async () => {
    try {
      [project, environments] = await Promise.all([
        projectsApi.list().then(list => list.find(p => p.id === projectId) ?? null),
        environmentsApi.list(projectId),
      ]);
      if (environments.length > 0) selectEnv(environments[0]);
    } catch (e: any) {
      error = e.message;
    } finally {
      loading = false;
    }
  });

  async function selectEnv(env: Environment) {
    selectedEnv = env;
    rawMode = false;
    rawDirty = false;
    varsLoading = true;
    try {
      const vars = await variablesApi.list(env.id);
      editingVars = vars.map(v => ({ key: v.key, value: v.value, masked: true, dirty: false }));
      rawText = toRaw(editingVars);
    } catch (e: any) {
      error = e.message;
    } finally {
      varsLoading = false;
    }
  }

  function toRaw(vars: { key: string; value: string }[]) {
    return vars.map(v => `${v.key}=${v.value}`).join('\n') + (vars.length ? '\n' : '');
  }

  function parseRaw(text: string): { key: string; value: string }[] | null {
    const result: { key: string; value: string }[] = [];
    for (const raw of text.split('\n')) {
      const line = raw.trim();
      if (!line || line.startsWith('#')) continue;
      const eq = line.indexOf('=');
      if (eq === -1) { rawError = `Invalid line: "${line}" — expected KEY=VALUE`; return null; }
      const key = line.slice(0, eq).trim();
      if (!key) { rawError = `Empty key in line: "${line}"`; return null; }
      const value = line.slice(eq + 1);
      result.push({ key, value });
    }
    return result;
  }

  function enterRaw() {
    rawText = toRaw(editingVars);
    rawDirty = false;
    rawError = '';
    rawMode = true;
  }

  function exitRaw() {
    rawMode = false;
    rawDirty = false;
    rawError = '';
  }

  async function saveRaw() {
    if (!selectedEnv) return;
    rawError = '';
    const parsed = parseRaw(rawText);
    if (!parsed) return;

    rawSaving = true;
    try {
      const incoming = new Map(parsed.map(v => [v.key, v.value]));
      const existing = new Map(editingVars.map(v => [v.key, v.value]));

      // Upsert added/changed
      const upserts = parsed.filter(v => existing.get(v.key) !== v.value);
      // Delete removed keys
      const deletions = editingVars.filter(v => !incoming.has(v.key));

      await Promise.all([
        ...upserts.map(v => variablesApi.upsert(selectedEnv!.id, v.key, v.value)),
        ...deletions.map(v => variablesApi.delete(selectedEnv!.id, v.key)),
      ]);

      editingVars = parsed.map(v => ({ key: v.key, value: v.value, masked: true, dirty: false }));
      rawText = toRaw(editingVars);
      rawDirty = false;
    } catch (e: any) {
      rawError = e.message;
    } finally {
      rawSaving = false;
    }
  }

  async function createEnv() {
    if (!newEnvName.trim()) return;
    addingEnv = true;
    try {
      const env = await environmentsApi.create(projectId, newEnvName.trim());
      environments = [...environments, env];
      newEnvName = '';
      selectEnv(env);
    } catch (e: any) {
      error = e.message;
    } finally {
      addingEnv = false;
    }
  }

  async function deleteEnv(env: Environment) {
    if (!confirm(`Delete environment "${env.name}"?`)) return;
    try {
      await environmentsApi.delete(projectId, env.id);
      environments = environments.filter(e => e.id !== env.id);
      if (selectedEnv?.id === env.id) {
        selectedEnv = null;
        editingVars = [];
        if (environments.length > 0) selectEnv(environments[0]);
      }
    } catch (e: any) {
      error = e.message;
    }
  }

  async function saveVar(idx: number) {
    if (!selectedEnv) return;
    const v = editingVars[idx];
    saving = true;
    try {
      await variablesApi.upsert(selectedEnv.id, v.key, v.value);
      editingVars[idx].dirty = false;
    } catch (e: any) {
      error = e.message;
    } finally {
      saving = false;
    }
  }

  async function deleteVar(key: string) {
    if (!selectedEnv || !confirm(`Delete variable "${key}"?`)) return;
    try {
      await variablesApi.delete(selectedEnv.id, key);
      editingVars = editingVars.filter(v => v.key !== key);
      rawText = toRaw(editingVars);
    } catch (e: any) {
      error = e.message;
    }
  }

  async function addVar() {
    if (!newKey.trim() || !selectedEnv) return;
    saving = true;
    try {
      await variablesApi.upsert(selectedEnv.id, newKey.trim(), newValue);
      editingVars = [...editingVars, { key: newKey.trim(), value: newValue, masked: true, dirty: false }];
      rawText = toRaw(editingVars);
      newKey = ''; newValue = '';
    } catch (e: any) {
      error = e.message;
    } finally {
      saving = false;
    }
  }
</script>

{#if loading}
  <div class="empty">Loading…</div>
{:else}
  <div class="breadcrumb">
    <a href="/projects">Projects</a>
    <span>/</span>
    <span>{project?.name ?? projectId}</span>
  </div>

  {#if error}
    <div class="error-msg" style="margin-bottom:16px">{error}</div>
  {/if}

  <div class="layout">
    <!-- Left: environments panel -->
    <aside class="card env-panel">
      <div class="panel-header">
        <span class="panel-title">Environments</span>
      </div>

      <div class="env-list">
        {#each environments as env}
          <div class="env-item" class:active={selectedEnv?.id === env.id}>
            <button class="env-btn" on:click={() => selectEnv(env)}>{env.name}</button>
            <button class="icon-btn btn-danger" on:click={() => deleteEnv(env)} title="Delete">✕</button>
          </div>
        {/each}
        {#if environments.length === 0}
          <div class="env-empty">No environments</div>
        {/if}
      </div>

      <form class="add-env-form" on:submit|preventDefault={createEnv}>
        <input
          type="text"
          bind:value={newEnvName}
          placeholder="e.g. production"
          disabled={addingEnv}
        />
        <button type="submit" class="btn-primary" disabled={addingEnv || !newEnvName.trim()}>
          {addingEnv ? '…' : '+'}
        </button>
      </form>
    </aside>

    <!-- Right: variables editor -->
    <div class="vars-section">
      {#if !selectedEnv}
        <div class="card empty-vars">Select or create an environment</div>
      {:else}
        <div class="vars-header">
          <span class="vars-title">
            Variables
            <span class="env-badge">{selectedEnv.name}</span>
          </span>
          <div class="view-toggle">
            <button
              class="toggle-btn"
              class:active={!rawMode}
              on:click={exitRaw}
              title="Table view"
            >Table</button>
            <button
              class="toggle-btn"
              class:active={rawMode}
              on:click={enterRaw}
              title="Raw .env editor"
            >Raw</button>
          </div>
        </div>

        {#if varsLoading}
          <div class="card empty">Loading variables…</div>
        {:else if rawMode}
          <!-- ── Raw editor ── -->
          <div class="card raw-card">
            <div class="raw-hint">Edit in <code>.env</code> format. Lines starting with <code>#</code> are ignored. Save applies additions, updates, and deletions.</div>
            {#if rawError}
              <div class="error-msg" style="margin-bottom:12px">{rawError}</div>
            {/if}
            <textarea
              bind:value={rawText}
              on:input={() => rawDirty = true}
              spellcheck="false"
              rows={Math.max(12, rawText.split('\n').length + 2)}
            ></textarea>
            <div class="raw-actions">
              <button class="btn-ghost" on:click={exitRaw}>Cancel</button>
              <button
                class="btn-primary"
                on:click={saveRaw}
                disabled={rawSaving || !rawDirty}
              >{rawSaving ? 'Saving…' : 'Save all'}</button>
            </div>
          </div>
        {:else}
          <!-- ── Table editor ── -->
          <div class="card vars-card">
            <table>
              <thead>
                <tr>
                  <th>Key</th>
                  <th>Value</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {#each editingVars as v, i}
                  <tr class:dirty={v.dirty}>
                    <td class="key-cell"><code>{v.key}</code></td>
                    <td class="val-cell">
                      <div class="val-wrap">
                        <input
                          type={v.masked ? 'password' : 'text'}
                          bind:value={v.value}
                          on:input={() => (editingVars[i].dirty = true)}
                        />
                        <button
                          type="button"
                          class="btn-ghost icon-btn reveal-btn"
                          on:click={() => (editingVars[i].masked = !v.masked)}
                          title={v.masked ? 'Reveal' : 'Mask'}
                        >{v.masked ? '👁' : '🙈'}</button>
                      </div>
                    </td>
                    <td class="actions-cell">
                      {#if v.dirty}
                        <button class="btn-primary icon-btn" on:click={() => saveVar(i)} disabled={saving}>✓</button>
                      {/if}
                      <button class="btn-danger icon-btn" on:click={() => deleteVar(v.key)}>✕</button>
                    </td>
                  </tr>
                {/each}
                <tr class="add-row">
                  <td><input type="text" bind:value={newKey} placeholder="NEW_KEY" /></td>
                  <td><input type="text" bind:value={newValue} placeholder="value" /></td>
                  <td>
                    <button class="btn-primary icon-btn" on:click={addVar} disabled={saving || !newKey.trim()}>+</button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        {/if}
      {/if}
    </div>
  </div>
{/if}

<style>
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: var(--muted);
    margin-bottom: 24px;
  }
  .breadcrumb a { color: var(--muted); }
  .breadcrumb a:hover { color: var(--text); }

  .layout {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 20px;
    align-items: start;
  }

  .env-panel { padding: 0; overflow: hidden; }

  .panel-header {
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
  }

  .panel-title { font-size: 12px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: var(--muted); }

  .env-list { padding: 6px; }

  .env-item {
    display: flex;
    align-items: center;
    border-radius: 6px;
  }
  .env-item.active { background: var(--bg3); }

  .env-btn {
    flex: 1;
    background: none;
    border: none;
    color: var(--text);
    font-size: 13px;
    text-align: left;
    padding: 7px 10px;
    border-radius: 6px;
  }
  .env-item.active .env-btn { color: var(--accent-hover); }

  .env-empty { padding: 12px; color: var(--muted); font-size: 12px; text-align: center; }

  .add-env-form {
    display: flex;
    gap: 6px;
    padding: 10px;
    border-top: 1px solid var(--border);
  }
  .add-env-form input { font-size: 12px; padding: 6px 8px; }
  .add-env-form button { padding: 6px 10px; flex-shrink: 0; }

  .vars-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .vars-title {
    font-size: 15px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .env-badge {
    background: var(--bg3);
    border: 1px solid var(--border);
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    padding: 2px 8px;
    color: var(--muted);
  }

  .view-toggle {
    display: flex;
    background: var(--bg3);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 2px;
    gap: 2px;
  }

  .toggle-btn {
    background: none;
    border: none;
    border-radius: 4px;
    color: var(--muted);
    font-size: 12px;
    font-weight: 500;
    padding: 4px 12px;
  }
  .toggle-btn.active {
    background: var(--bg2);
    color: var(--text);
    box-shadow: 0 1px 3px rgba(0,0,0,0.3);
  }

  /* Raw editor */
  .raw-card { display: flex; flex-direction: column; gap: 12px; }

  .raw-hint {
    font-size: 12px;
    color: var(--muted);
    line-height: 1.5;
  }
  .raw-hint code {
    background: var(--bg3);
    border-radius: 3px;
    padding: 1px 5px;
    font-size: 11px;
    color: var(--accent-hover);
  }

  textarea {
    background: var(--bg3);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    color: var(--text);
    font-family: 'JetBrains Mono', 'Fira Code', 'Menlo', monospace;
    font-size: 13px;
    line-height: 1.7;
    padding: 12px 14px;
    resize: vertical;
    width: 100%;
    outline: none;
  }
  textarea:focus { border-color: var(--accent); }

  .raw-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  /* Table editor */
  .vars-card { padding: 0; overflow: hidden; }

  table { width: 100%; border-collapse: collapse; }

  th {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
    padding: 10px 14px;
    text-align: left;
    border-bottom: 1px solid var(--border);
  }

  td { padding: 8px 14px; border-bottom: 1px solid var(--border); vertical-align: middle; }
  tr:last-child td { border-bottom: none; }
  tr.dirty { background: rgba(99, 102, 241, 0.06); }

  .key-cell { width: 200px; }
  code { font-family: monospace; font-size: 12px; color: var(--accent-hover); }

  .val-wrap { display: flex; align-items: center; gap: 6px; }
  .reveal-btn { padding: 5px 7px; font-size: 12px; }

  .actions-cell {
    width: 80px;
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .icon-btn { padding: 5px 8px; font-size: 11px; }
  .add-row td { background: var(--bg); }

  .empty { color: var(--muted); padding: 24px; text-align: center; }
  .empty-vars { color: var(--muted); text-align: center; padding: 40px; }
</style>
