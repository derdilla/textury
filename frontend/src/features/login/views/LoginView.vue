<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router';
import OrbitArtwork from '@/features/login/components/OrbitArtwork.vue';

import logo_dark from '@/shared/assets/logo_dark.svg';
import logo_light from '@/shared/assets/logo_light.svg';
</script>

<template>
  <div class="auth-layout">
    <!-- Brand Panel -->
    <div class="brand-panel middle-align center-align large-padding">
      <div class="artwork-wrapper middle-align center-align">
        <OrbitArtwork />
      </div>
      <img :src="logo_dark" alt="Logo" class="logo-mobile responsive dark-only" />
      <img :src="logo_light" alt="Logo" class="logo-mobile responsive light-only" />
    </div>

    <!-- Auth Panel -->
    <div class="form-panel surface-container">
      <div class="form-content padding">
        <div class="tabs no-round no-border small bottom-margin top-padding">
          <RouterLink class="link transparent" activeClass="active secondary-container" to="/login">
            <i>lock_open</i>
            <span class="tab-text">Sign In</span>
          </RouterLink>
          <RouterLink class="link transparent" activeClass="active secondary-container" to="/register">
            <i>person_add</i>
            <span class="tab-text">Register</span>
          </RouterLink>
          <RouterLink class="link transparent" activeClass="active secondary-container" to="/joinSingleProject">
            <i>link_2</i>
            <span class="tab-text">Join Project</span>
          </RouterLink>
        </div>
        <hr v-if="$route.matched.length > 1" style="max-width: 410px;" class="bottom-margin">
        <RouterView/>
      </div>
    </div>
  </div>
</template>

<style scoped>
.auth-layout {
  display: flex;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.dark-only { display: none; }
.light-only { display: block; }

.brand-panel {
  width: 50%;
  height: 100%;
  overflow: hidden;
}

.artwork-wrapper {
  width: 540px;
  height: 540px;
  flex-shrink: 0;
  transform: scale(min(1, calc(35vw / 540px), calc(70vh / 540px)));
  transform-origin: center center;
}

.logo-mobile {
  max-width: 220px;
}

@media (min-width: 891px) {
  .logo-mobile {
    display: none !important;
  }
}

.form-panel {
  width: 50%;
  height: 100vh;
  border-radius: 18px 0 0 18px;
}

.form-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  overflow-y: auto;
  overflow-x: hidden;
}

.tabs {
  max-width: 410px;
  width: 100%;
  gap: 0.5rem;
}

.link::before {
  background-color: transparent !important;
}

.link.active i {
  color: var(--on-secondary-container);
  font-variation-settings: 'FILL' 0;
}

.tab-text {
  margin-left: 0.5rem;
}

@media (max-width: 890px) {
  .auth-layout {
    flex-direction: column;
    overflow-y: auto;
  }

  .brand-panel {
    width: 100%;
    height: auto;
    padding: 2.5rem 1rem 1.5rem 1rem;
  }

  .artwork-wrapper {
    display: none;
  }

  .form-panel {
    width: 100%;
    min-height: calc(100vh - 140px);
    flex: 1;
    border-radius: 24px 24px 0 0;
  }

  .form-content {
    justify-content: center;
  }
}

.dark .dark-only { display: block; }
.dark .light-only { display: none; }
</style>
