<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '@/shared/stores/authStore'

const authStore = useAuthStore();

const props = defineProps<{
  shareId: string;
}>();

const shareId = ref(props.shareId);
const password = ref('');
const isLoading = ref(false);
const errorMessage = ref('');


async function handleSubmit() {
  if (!shareId.value || !password.value || isLoading.value) return;

  isLoading.value = true;
  errorMessage.value = '';

  try {
    const res = await authStore.joinProject({ shareId: shareId.value, password: password.value });
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
        <input v-model="shareId" autocomplete="off" required>
        <label>Share ID</label>
      </div>

      <div class="field label border no-round small vertical-margin">
        <input v-model="password" type="password" autocomplete="off" autofocus required>
        <label>Password</label>
      </div>
    </div>

    <button type="submit" :disabled="isLoading"
      class="no-round submit-button top-margin no-padding">
      <i>arrow_forward</i>
      <span>{{ isLoading ? 'Opening...' : 'Open Project' }}</span>
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
