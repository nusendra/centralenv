import { getToken, setToken } from './auth';

// Empty string = same origin (Vite dev proxy or single-domain prod setup).
// Set VITE_API_URL at build time to point at a separate API host.
const BASE = (import.meta.env.VITE_API_URL ?? '').replace(/\/$/, '');

async function req<T>(method: string, path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = {};
  if (body) headers['Content-Type'] = 'application/json';
  const token = getToken();
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${BASE}${path}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });

  if (res.status === 401) throw new Error('UNAUTHORIZED');
  if (!res.ok) {
    const err = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(err.error ?? res.statusText);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

// Auth
export const authApi = {
  login: async (username: string, password: string) => {
    const res = await req<{ username: string; token: string }>('POST', '/auth/login', { username, password });
    setToken(res.token);
    return res;
  },
  logout: () => req<void>('POST', '/auth/logout'),
  me: () => req<void>('GET', '/auth/me'),
};

// Projects
export interface Project { id: string; name: string; slug: string; created_at: string }
export const projectsApi = {
  list: () => req<Project[]>('GET', '/api/projects'),
  create: (name: string, slug: string) => req<Project>('POST', '/api/projects', { name, slug }),
  update: (id: string, name: string, slug: string) => req<Project>('PUT', `/api/projects/${id}`, { name, slug }),
  delete: (id: string) => req<void>('DELETE', `/api/projects/${id}`),
};

// Environments
export interface Environment { id: string; project_id: string; name: string; created_at: string }
export const environmentsApi = {
  list: (projectId: string) => req<Environment[]>('GET', `/api/projects/${projectId}/environments`),
  create: (projectId: string, name: string) =>
    req<Environment>('POST', `/api/projects/${projectId}/environments`, { name }),
  delete: (projectId: string, envId: string) =>
    req<void>('DELETE', `/api/projects/${projectId}/environments/${envId}`),
};

// Variables
export interface Variable { id: string; key: string; value: string; updated_at: string }
export const variablesApi = {
  list: (envId: string) => req<Variable[]>('GET', `/api/environments/${envId}/variables`),
  upsert: (envId: string, key: string, value: string) =>
    req<void>('POST', `/api/environments/${envId}/variables`, { key, value }),
  delete: (envId: string, key: string) =>
    req<void>('DELETE', `/api/environments/${envId}/variables/${key}`),
};

// Tokens
export interface Token { id: string; name: string; project_ids: string[]; last_used_at: string | null; created_at: string }
export interface TokenCreated extends Token { token: string }
export const tokensApi = {
  list: () => req<Token[]>('GET', '/api/tokens'),
  create: (name: string, project_ids: string[]) =>
    req<TokenCreated>('POST', '/api/tokens', { name, project_ids }),
  delete: (id: string) => req<void>('DELETE', `/api/tokens/${id}`),
};
