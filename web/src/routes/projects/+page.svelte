<script lang="ts">
  import { onMount } from 'svelte';
  import { projectsApi, type Project } from '$lib/api';

  let projects: Project[] = [];
  let loading = true;
  let error = '';
  let showForm = false;
  let newName = '';
  let newSlug = '';
  let saving = false;

  onMount(load);

  async function load() {
    try {
      projects = await projectsApi.list();
    } catch (e: any) {
      error = e.message;
    } finally {
      loading = false;
    }
  }

  function slugify(s: string) {
    return s.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
  }

  async function createProject() {
    if (!newName || !newSlug) return;
    saving = true;
    try {
      const p = await projectsApi.create(newName, newSlug);
      projects = [...projects, p];
      newName = ''; newSlug = ''; showForm = false;
    } catch (e: any) {
      error = e.message;
    } finally {
      saving = false;
    }
  }

  async function deleteProject(p: Project) {
    if (!confirm(`Delete project "${p.name}"? This will remove all environments and variables.`)) return;
    try {
      await projectsApi.delete(p.id);
      projects = projects.filter(x => x.id !== p.id);
    } catch (e: any) {
      error = e.message;
    }
  }
</script>

<div class="page-header">
  <h2>Projects</h2>
  <button class="btn-primary" on:click={() => showForm = !showForm}>
    {showForm ? 'Cancel' : '+ New project'}
  </button>
</div>

{#if error}
  <div class="error-msg" style="margin-bottom:16px">{error}</div>
{/if}

{#if showForm}
  <div class="card new-form">
    <h3>New project</h3>
    <form on:submit|preventDefault={createProject}>
      <div class="row">
        <label>
          Name
          <input
            type="text"
            bind:value={newName}
            on:input={() => newSlug = slugify(newName)}
            placeholder="My App"
            required
          />
        </label>
        <label>
          Slug
          <input type="text" bind:value={newSlug} placeholder="my-app" required />
        </label>
      </div>
      <div class="form-actions">
        <button type="submit" class="btn-primary" disabled={saving}>
          {saving ? 'Creating…' : 'Create project'}
        </button>
      </div>
    </form>
  </div>
{/if}

{#if loading}
  <div class="empty">Loading…</div>
{:else if projects.length === 0}
  <div class="empty card">
    <p>No projects yet. Create your first project above.</p>
  </div>
{:else}
  <div class="project-grid">
    {#each projects as p}
      <div class="card project-card">
        <div class="project-top">
          <div>
            <div class="project-name">{p.name}</div>
            <div class="project-slug">{p.slug}</div>
          </div>
          <button class="btn-danger icon-btn" on:click={() => deleteProject(p)} title="Delete">✕</button>
        </div>
        <a href="/projects/{p.id}" class="btn-ghost open-btn">Open →</a>
      </div>
    {/each}
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

  .new-form {
    margin-bottom: 24px;
  }

  .new-form h3 {
    font-size: 15px;
    font-weight: 600;
    margin-bottom: 16px;
  }

  .row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .form-actions {
    margin-top: 14px;
    display: flex;
    justify-content: flex-end;
  }

  .project-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 14px;
  }

  .project-card {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .project-top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
  }

  .project-name { font-size: 15px; font-weight: 600; }
  .project-slug { font-size: 12px; color: var(--muted); margin-top: 2px; font-family: monospace; }

  .icon-btn { padding: 5px 8px; font-size: 11px; }

  .open-btn {
    display: block;
    text-align: center;
    text-decoration: none;
    color: var(--text);
  }

  .empty {
    color: var(--muted);
    text-align: center;
    padding: 40px 20px;
  }
</style>
