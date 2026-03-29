<script setup lang="ts">
import {RouterLink, RouterView} from 'vue-router'
import AboutApp from '../components/AboutApp.vue'
import {whoami} from "@/services/managementApi.ts";
import router from "@/router";
import { editCtrl } from '@/services/editorController';

whoami().then(async (user) => {
  editCtrl.user = user;
  if (user !== null) await router.push('/projects');
});
</script>

<template>
  <div class="grid large-padding auto-width center"
       style="max-width: 1400px;">
    <div class="s12 m12 l6">
      <header class="middle">
        <AboutApp/>

        <nav class="tabbed">
          <RouterLink activeClass="active" to="/register">Register</RouterLink>
          <RouterLink activeClass="active" to="/login">Login</RouterLink>
        </nav>
      </header>
    </div>


    <div class="s12 m12 l6">
      <RouterView/>
    </div>

  </div>
</template>
