<script setup>
import { editCtrl } from '@/services/editorController.ts';
import {login} from '../services/managementApi.ts'
import router from "@/router/index.ts";

const email = defineModel('email')
const password = defineModel('password')

async function loginClicked() {
  const success = await login({
    email: email.value,
    password: password.value,
  });
  if (success) {
    editCtrl.user = success;
    await router.push('/projects')
  }
}
</script>

<template>
  <form class="large-padding middle" @submit.prevent>
    <div class="field label border">
      <input v-model="email" type="text">
      <label>E-Mail</label>
    </div>
    <div class="field label border">
      <input v-model="password" type="password" minlength="8">
      <label>Password</label>
      <output class="invalid">Must be at least 8 characters</output>
    </div>
    <div class="row">
      <div class="max"></div>
      <button class="large medium-margin" @click="loginClicked"><i>login</i><span>Login</span></button>
    </div>
  </form>
</template>
