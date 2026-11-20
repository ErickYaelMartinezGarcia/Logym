import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

// Demo web aislada: las importaciones nativas apuntan a implementaciones JS.
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@tauri-apps/api/tauri': fileURLToPath(new URL('./src/demo/tauriMock.js', import.meta.url)),
      '@tauri-apps/api/event': fileURLToPath(new URL('./src/demo/eventMock.js', import.meta.url))
    }
  }
})
