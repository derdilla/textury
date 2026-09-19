<script setup>
import {register} from '@/services/api/managementApi'
import router from "@/router/index.ts";

const token = defineModel('token')
const name = defineModel('name')
const email = defineModel('email')
const password = defineModel('password')

async function registerClicked() {

  const success = await register({
    registration_token: token.value,
    username: name.value,
    email: email.value,
    password: password.value,
  });
  if (success) {
    await router.push('/projects')
  }
}
</script>

<template>
  <form class="" @submit.prevent>
    <div class="fields">
      <div class="field label border no-round small">
        <input v-model="token" type="text">
        <label>Registration token</label>
        <output class="invalid">Invalid or expired registration token</output>
      </div>
      <div class="field label border no-round small">
        <input v-model="name" type="text" minlength="4">
        <label>Username</label>
        <output class="invalid">Must be at least 4 characters</output>
      </div>
      <div class="field label border no-round small">
        <input v-model="email" type="text">
        <label>E-Mail</label>
      </div>
      <div class="field label border no-round small vertical-margin">
        <input v-model="password" type="password" minlength="8">
        <label>Password</label>
        <output class="invalid">Must be at least 8 characters</output>
      </div>
    </div>
    <button class="no-round submit-button top-margin no-padding" @click="registerClicked"><i>add</i><span>Create Account</span></button>
  </form>
</template>

<style scoped>
.submit-button {
  width: 100%;
}
form {
  max-width: 400px;
  width: 100%;
}
.fields {
  min-height: 230px;
}
</style>