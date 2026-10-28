<script setup lang="ts">
import { ref, watch } from 'vue'
import { RouterView } from 'vue-router'
import { useAuthStore } from '@/shared/stores/authStore'

const authStore = useAuthStore()
const showLoader = ref(false)
let timer: ReturnType<typeof setTimeout> | null = null

watch(
  () => !authStore.state.isInitialized || authStore.state.isLoading,
  (isLoading) => {
    if (isLoading) {
      timer = setTimeout(() => {
        showLoader.value = true
      }, 250) // Delay to mitigate a very short appearance of the loading spinner when requests are fast.
    } else {
      if (timer) {
        clearTimeout(timer)
        timer = null
      }
      showLoader.value = false
    }
  },
  { immediate: true }
)
</script>

<template>
  <div v-if="showLoader" class="app-loader">
    <progress class="circle large"></progress>
  </div>

  <RouterView v-else />
</template>

<style>
.app-loader {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: var(--surface);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 99999;
}

*:not(i):not(:has(.cm-editor *)):not(.cm-editor *):not(.CodeMirror *):not(.token-text) {
  font-family: Avenir, Montserrat, Corbel, 'URW Gothic', source-sans-pro, sans-serif !important;
}

.cm-editor, .cm-editor *, .CodeMirror, .CodeMirror *, .token-text {
  font-family: 'SF Mono', 'Segoe UI Mono', 'Input Mono', 'Roboto Mono', Monaco, Consolas, monospace !important;
  font-weight: 400 !important;

  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.tooltip {
  border-radius: 8px !important;
}

html, body {
  background-color: var(--surface);
}
</style>
