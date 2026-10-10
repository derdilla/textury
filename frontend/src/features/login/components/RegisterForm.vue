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
  <form @submit.prevent="handleSubmit">
    <div class="fields">
      <div v-if="errorMessage" class="chip no-round error-text row medium no-padding">
        <i class="large-padding left-padding">error</i>
        <span class="large-padding right-padding">{{ errorMessage }}</span>
      </div>

      <div v-if="needsToken" class="field label border no-round small">
        <input v-model="token" type="text" required>
        <label>Registration Token</label>
        <output class="invalid">Invalid or expired registration token</output>
      </div>

      <div class="field label border no-round small">
        <input v-model="username" type="text" minlength="4" required>
        <label>Username</label>
        <output class="invalid">Must be at least 4 characters</output>
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
  max-width: 400px;
  width: 100%;
}
.fields {
  min-height: 280px;
}
</style>
