<script setup>
import { editCtrl } from '@/services/editorController.ts';
import {login} from '@/services/api/managementApi'
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
  <form class="" @submit.prevent>
    <div class="fields">
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
    <button class="no-round submit-button top-margin no-padding" @click="loginClicked"><i>arrow_forward</i><span>Sign In</span></button>
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