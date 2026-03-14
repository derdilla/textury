<script setup>
import {register} from '../services/managementApi.ts'
import {ref} from "vue";
import router from "@/router/index.ts";

const token = defineModel('token')
const name = defineModel('name')
const email = defineModel('email')
const password = defineModel('password')
const password2 = defineModel('password2')

const pwMismatch = ref(false);

async function registerClicked() {
  pwMismatch.value = (password.value !== password2.value);
  if (pwMismatch.value) {
    console.warn('pw-missmatch')
    return;
  }

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
      <label>Registration token</label>
      <input v-model="token" type="text">
      <output class="invalid">Invalid or expired registration token</output>
    </div>
    <div class="field label border">
      <label>Name</label>
      <input v-model="name" type="text" minlength="4">
      <output class="invalid">Must be at least 4 characters</output>
    </div>
    <div class="field label border">
      <label>E-Mail</label>
      <input v-model="email" type="text">
    </div>
    <div class="field label border">
      <label>Password</label>
      <input v-model="password" type="password" minlength="8">
      <output class="invalid">Must be at least 8 characters</output>
    </div>
    <div class="field label border" :class="{invalid: pwMismatch}">
      <label>Repeat password</label>
      <input v-model="password2" type="password" minlength="8">
      <output class="invalid">Passwords don't match</output>
    </div>
    <div class="row">
      <div class="max"></div>
      <button @click="registerClicked" class="large medium-margin"><i>login</i><span>Register</span></button>
    </div>
  </form>
</template>
