<script setup lang="ts">
import {RouterView} from 'vue-router'
</script>

<template>
  <RouterView/>
</template>

<style>
*:not(i) {
  font-family: Avenir, Montserrat, Corbel, 'URW Gothic', source-sans-pro, sans-serif !important;
}

.tooltip {
  border-radius: 8px !important;
}

html, body {
  background-color: var(--surface);
}
</style>