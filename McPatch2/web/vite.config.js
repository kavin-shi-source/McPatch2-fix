import {fileURLToPath, URL} from 'node:url'

import {defineConfig} from 'vite'
import react from '@vitejs/plugin-react'

import { getManualChunkName } from './build/manualChunks.js'

// https://vite.dev/config/
export default defineConfig({
  base: '/',
  plugins: [
    react()
  ],
  build: {
    rollupOptions: {
      output: {
        manualChunks: getManualChunkName
      }
    }
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  }
})
