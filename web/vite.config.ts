import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const API_URL = (import.meta as any).env?.VITE_API_URL ?? 'http://localhost:3001';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      '/auth': { target: API_URL, changeOrigin: true },
      '/api':  { target: API_URL, changeOrigin: true },
    }
  }
});
