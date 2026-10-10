<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAuthStore } from '@/shared/stores/authStore'
import { useAccountSettingsStore } from '@/features/settings/stores/accountSettingsStore';

const authStore = useAuthStore();
const accountSettingsStore = useAccountSettingsStore();
accountSettingsStore.fetchServerSettings();

const token = ref('');
const username = ref('');
const email = ref('');
const password = ref('');
const isLoading = ref(false);
const errorMessage = ref('');

const needsToken = computed(() => {
  if (accountSettingsStore.state.serverSettings?.openSignUps) return false;
  return true;
})

async function handleSubmit() {
  if (!token.value && needsToken.value) return;
  if (!username.value || !email.value || !password.value || isLoading.value) return;

  isLoading.value = true;
  errorMessage.value = '';

  try {
    const res = await authStore.register({
      registrationToken: token.value || 'no token needed',
      username: username.value,
      email: email.value,
      password: password.value,
    });
    if (!res.success && res.error) {
      errorMessage.value = res.error;
    }
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <form @submit.prevent="handleSubmit" autocomplete="on">
    <div class="fields">
      <div v-if="errorMessage" class="chip max no-round error-text grid bottom-margin error-box">
        <i class="s1">error</i>
        <span class="s11 wrap text-wrap">{{ errorMessage }}</span>
      </div>

      <div v-if="needsToken" class="field label border no-round small form-box">
        <input v-model="token" type="text" autocomplete="one-time-code" required class="form-input">
        <label class="form-label">Registration Token</label>
        <output class="invalid">Invalid or expired registration token</output>
      </div>

      <div class="field label border no-round small form-box">
        <input v-model="username" name="display_name" type="text" autocomplete="off" minlength="4" required class="form-input">
        <label class="form-label">Username</label>
        <output class="invalid">Must be at least 4 characters</output>
      </div>

      <div class="field label border no-round small form-box">
        <input v-model="email" name="email" type="email" autocomplete="username" required class="form-input">
        <label class="form-label">E-Mail</label>
      </div>

      <div class="field label border no-round small vertical-margin form-box">
        <input v-model="password" name="password" type="password" autocomplete="new-password" minlength="8" required class="form-input password-input">
        <label class="form-label">Password</label>
        <output class="invalid">Must be at least 8 characters</output>
        <i class="front">visibility</i>
      </div>
    </div>

    <button type="submit" :disabled="isLoading"
      class="no-round submit-button top-margin no-padding">
      <i>add</i>
      <span>{{ isLoading ? 'Creating Account...' : 'Create Account' }}</span>
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
