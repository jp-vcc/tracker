import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

// Fixed dev port: src-tauri/tauri.conf.json `devUrl` must match.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: { target: 'chrome105', outDir: 'dist' },
  test: { globals: true, environment: 'jsdom' },
});

