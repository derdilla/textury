import { fileURLToPath } from 'url'
import { defineConfig } from 'vitest/config'
import { playwright } from '@vitest/browser-playwright'
import vue from '@vitejs/plugin-vue'
import wasm from 'vite-plugin-wasm'


export default defineConfig({
  plugins: [
    vue(),
    wasm(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
  server: {
    proxy: {
      '/api/v1': {
        target: 'http://127.0.0.1:3000',
        secure: false,
        changeOrigin: true,
        ws: true,
      },
      '/testGetInitialRegistrationToken': {
        target: 'http://127.0.0.1:3000',
      }
    }
  },
  test: {
    globalSetup: [
      './tests/setup.ts',
    ],
    include: [
      './tests/*.test.ts',
    ],
    browser: {
      provider: playwright(),
      enabled: true,
      instances: [
        { browser: 'chromium' },
      ],
    },
  },
})
