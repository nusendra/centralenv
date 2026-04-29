import { writable } from 'svelte/store';

const TOKEN_KEY = 'centralenv_token';

function initialToken(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem(TOKEN_KEY);
}

function initialAuthed(): boolean | null {
  return initialToken() ? true : null;
}

export const authed = writable<boolean | null>(initialAuthed());

export function getToken(): string | null {
  return initialToken();
}

export function setToken(token: string) {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(TOKEN_KEY, token);
  }
  authed.set(true);
}

export function clearToken() {
  if (typeof localStorage !== 'undefined') {
    localStorage.removeItem(TOKEN_KEY);
  }
  authed.set(false);
}

export function setAuthed(value: boolean) {
  if (!value) clearToken();
  else authed.set(true);
}
