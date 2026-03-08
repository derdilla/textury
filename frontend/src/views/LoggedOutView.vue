<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import AboutApp from '../components/AboutApp.vue'
</script>

<template>
  <header>
    <AboutApp />

    <nav>
      <RouterLink to="/register">Register</RouterLink>
      <RouterLink to="/login">Login</RouterLink>
    </nav>
  </header>

  <RouterView />
</template>
