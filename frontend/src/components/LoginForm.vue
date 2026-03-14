<script setup>
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
    await router.push('/projects')
  }
}
</script>

<template>
  <form class="large-padding middle" @submit.prevent>
    <div class="field label border">
      <label>E-Mail</label>
      <input v-model="email" type="text">
    </div>
    <div class="field label border">
      <label>Password</label>
      <input v-model="password" type="password" minlength="8">
      <output class="invalid">Must be at least 8 characters</output>
    </div>
    <div class="row">
      <div class="max"></div>
      <button class="large medium-margin" @click="loginClicked"><i>login</i><span>Login</span></button>
    </div>
  </form>
</template>