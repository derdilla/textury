<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '@/shared/stores/authStore'

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
      <div v-if="errorMessage" class="chip max no-round error-text grid bottom-margin error-box">
        <i class="s1">error</i>
        <span class="s11 wrap text-wrap">{{ errorMessage }}</span>
      </div>

      <div class="field label border no-round small form-box">
        <input v-model="email" type="email" required class="form-input">
        <label class="form-label">E-Mail</label>
      </div>

      <div class="field label suffix border no-round small vertical-margin form-box">
        <input v-model="password" type="password" minlength="8" required class="form-input password-input">
        <label class="form-label">Password</label>
        <output class="invalid">Must be at least 8 characters</output>
        <i class="front">visibility</i>
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
  max-width: 410px;
  width: 100%;
}
.fields {
  min-height: 280px;
}

.form-input {
  /* Overwrite default BeerCSS paddings that come with prefix and suffix. */
  padding-inline-start: 1rem !important;
  padding-inline-end: 1rem !important;
}
.password-input {
  /* Overwrite default BeerCSS paddings that come with prefix and suffix. */
  padding-inline-start: 1rem !important;
  padding-inline-end: 3rem !important;
}
.form-input:is(:focus,[placeholder]:not(:placeholder-shown),.active) {
  /* Slightly changed mask from BeerCSS. Changed 1.5rem to 0.75rem. */
  clip-path: polygon(-2% -2%, 0.75rem -2%, 0.75rem .5rem, calc(100% - 2rem) .5rem, calc(100% - 2rem) -2%, 102% -2%, 102% 102%, -2% 102%) !important;
}
.form-input:focus {
  /* Offset of 0.0625 is needed to stop the text from jumping when focusing. Derived from BeerCSS. */
  padding-inline-start: 0.9375rem !important;
}
.form-box label {
  margin-left: -0.5rem !important;
}
.form-label {
  margin-left: -0.5rem !important;
}

.error-box {
  overflow: hidden;
  padding-top: 0.6rem;
  padding-bottom: 0.6rem;
}
</style>
