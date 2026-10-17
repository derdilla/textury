<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useProjectPublicShareStore } from "../stores/projectPublicShareStore";
import { useAccountSettingsStore } from "@/features/settings/stores/accountSettingsStore";

const props = defineProps<{
  projectId: string;
}>();

const store = computed(() => useProjectPublicShareStore(props.projectId));

const shareId = ref('');
const password = ref('');
const isLoading = ref(false);
const errorMessage = ref('');


watch(store, (store) => {
  if (shareId.value.length < 1) {
    shareId.value = store.state.value.shareId;
  }
  if (password.value.length < 1) {
    password.value = store.state.value.password;
  }
}, { immediate: true, deep: true })

const url = computed<null | string>(() => {
  if (!store.value.state.value.isShared) return null;
  return `${window.location.origin}/joinSingleProject/${store.value.state.value.shareId}`;
});


const accountSettingsStore = useAccountSettingsStore();
const canShare = computed(() => {
  if (accountSettingsStore.state.serverSettings?.openSharing) return true;
  return false;
})

function saveInputs(): boolean {
  let res = store.value.setId(shareId.value);
  if (res) {
    errorMessage.value = res;
    return false;
  }
  res = store.value.setPassword(password.value);
  if (res) {
    errorMessage.value = res;
    return false;
  }
  return true;
}

async function handleSubmit() {
  try {
    isLoading.value = true;

    if (store.value.state.value.isShared) {
      await store.value.unshare();
    } else {
      if (!saveInputs()) return;
      const error = await store.value.share();
      if (error) {
        errorMessage.value = error;
      }
    }
  } finally {
    isLoading.value = false;
  }
}

function clearError() {
  errorMessage.value = '';
}

function copyUrl() {
  if (!url.value) return;
  navigator.clipboard.writeText(url.value)
}

</script>

<template>
  <form v-if="canShare || store.state.value.isShared" @submit.prevent="handleSubmit" class="share-project-publicly-form">
    <div class="fields">
      <div v-if="errorMessage" class="chip max no-round error-text grid bottom-margin error-box" @click="clearError">
        <i class="s1">error</i>
        <span class="s11 wrap text-wrap">{{ errorMessage }}</span>
      </div>

      <div v-if="url" class="chip no-round row medium no-padding" @click="copyUrl">
        <a class="left-padding max left-align">{{ url }}</a>
        <i class="right-padding">content_copy</i>
      </div>

      <div class="field label border no-round small form-box">
        <input v-model="shareId"
            minlength="2"
            @blur="saveInputs"
            autocomplete="off"
            :readonly="store.state.value.isShared"
            required
            class="form-input">
        <label class="form-label">Share ID</label>
        <output class="invalid">Must be at least 2 characters</output>
      </div>

      <div class="field label border no-round small vertical-margin form-box">
        <input v-model="password"
            type="password"
            minlength="8"
            @blur="saveInputs"
            autocomplete="new-password"
            :readonly="store.state.value.isShared"
            required
            class="form-input password-input">
        <label class="form-label">Password</label>
        <output class="invalid">Must be at least 8 characters</output>
      </div>
    </div>

    <button type="submit" :disabled="isLoading"
      class="no-round submit-button top-margin no-padding">
      <i>arrow_forward</i>
      <span v-if="!store.state.value.isShared">{{ isLoading ? 'Sharing...' : 'Share Now' }}</span>
      <span v-else>{{ isLoading ? 'Unsharing...' : 'Unshare & Kick temporary users' }}</span>
    </button>
  </form>
</template>

<style scoped>
.share-project-publicly-form {
  width: 100%;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 2em;
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
