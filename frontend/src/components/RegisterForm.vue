<script setup>
import {register} from '../services/managementApi.ts'
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
  <form class="large-padding middle" @submit.prevent>
    <div class="field label border">
      <input v-model="token" type="text">
      <label>Registration token</label>
      <output class="invalid">Invalid or expired registration token</output>
    </div>
    <div class="field label border">
      <input v-model="name" type="text" minlength="4">
      <label>Name</label>
      <output class="invalid">Must be at least 4 characters</output>
    </div>
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
      <button @click="registerClicked" class="large medium-margin"><i>login</i><span>Register</span></button>
    </div>
  </form>
</template>
