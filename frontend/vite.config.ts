import {defineConfig} from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 4173,
    proxy: {'/v1': 'http://127.0.0.1:18080'},
  },
  preview: {port: 4173},
});
