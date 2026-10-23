<script setup lang="ts">
import {RouterView} from 'vue-router'
</script>

<template>
  <RouterView/>
</template>

<style>
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
