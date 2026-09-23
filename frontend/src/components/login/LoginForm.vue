<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore();

const email = ref('');
const password = ref('');
const isLoading = ref(false);
const errorMessage = ref('');

async function handleSubmit() {
  if (!email.value || !password.value || isLoading.value) return;

  isLoading.value = true;
  errorMessage.value = '';

  try {
    const res = await authStore.login({ email: email.value, password: password.value });
    if (!res.success && res.error) {
      errorMessage.value = res.error;
    }
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <form @submit.prevent="handleSubmit">
    <div class="fields">
      <div v-if="errorMessage" class="chip no-round error-text row medium no-padding">
        <i class="large-padding left-padding">error</i>
        <span class="large-padding right-padding">{{ errorMessage }}</span>
      </div>

      <div class="field label border no-round small">
        <input v-model="email" type="email" required>
        <label>E-Mail</label>
      </div>

      <div class="field label border no-round small vertical-margin">
        <input v-model="password" type="password" minlength="8" required>
        <label>Password</label>
        <output class="invalid">Must be at least 8 characters</output>
      </div>
    </div>

    <button type="submit" :disabled="isLoading"
      class="no-round submit-button top-margin no-padding">
      <i>arrow_forward</i>
      <span>{{ isLoading ? 'Signing In...' : 'Sign In' }}</span>
    </button>
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
  min-height: 280px;
}
</style>