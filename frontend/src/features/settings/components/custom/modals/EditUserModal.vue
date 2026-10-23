<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";
import CodeOutputField from "@/shared/components/composites/CodeOutputField.vue";
import type { PrivateAccountInfo } from "@/api/types";

const open = defineModel<boolean>("open", { default: false });
const props = defineProps<{
  user: PrivateAccountInfo | null;
}>();

const emit = defineEmits<{
  (e: "save", data: { username: string; email: string }): void;
}>();

const username = ref("");
const email = ref("");
const userId = ref("");
const usernameInputField = ref<HTMLInputElement | null>(null);

watch(open, async (isOpen) => {
  if (isOpen && props.user) {
    username.value = props.user.username;
    email.value = props.user.email;
    userId.value = props.user.id;
    await nextTick();
    usernameInputField.value?.focus();
  }
}, { immediate: true });

function submit() {
  if (!username.value.trim() || !email.value.trim()) return;
  emit("save", { username: username.value, email: email.value });
  open.value = false;
}
</script>

<template>
  <ModalPopover v-model:open="open" max-width="500px">
    <template #header>
      <div class="max dialog-header-content">
        <h5 class="bold dialog-title no-margin">Edit User</h5>
      </div>
    </template>

    <div class="modal-body-content" @keydown.enter.prevent="submit">
      <div class="field label border no-round small bottom-margin input-field user-input-box">
        <input ref="usernameInputField" v-model="username" type="text" class="user-input active" placeholder=" ">
        <label class="user-label active">Username</label>
      </div>

      <div class="field label border no-round small bottom-margin input-field user-input-box">
        <input v-model="email" type="email" class="user-input active" placeholder=" ">
        <label class="user-label active">Email</label>
      </div>

      <CodeOutputField
        :value="userId"
        label="User ID"
        :show-share="false"
      />
    </div>

    <template #actions>
      <div class="row gap">
        <button class="border no-round" @click="open = false">Cancel</button>
        <button class="primary no-round" @click="submit" :disabled="!username.trim() || !email.trim()" :class="{ 'is-disabled': !username.trim() || !email.trim() }">
          <i>check</i> Save
        </button>
      </div>
    </template>
  </ModalPopover>
</template>

<style scoped>
.dialog-header-content {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.15rem;
  width: 100%;
  min-width: 0;
}

.modal-body-content {
  display: flex;
  flex-direction: column;
  width: 100%;
  margin-top: 0.5rem;
  margin-bottom: 1rem;
}

.input-field {
  width: 100% !important;
}

.is-disabled, button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  pointer-events: none;
}

.user-input {
  padding-inline-start: 1rem !important;
  padding-inline-end: 1rem !important;
}

.user-input:is(:focus,[placeholder]:not(:placeholder-shown),.active) {
  clip-path: polygon(-2% -2%, 0.75rem -2%, 0.75rem .5rem, calc(100% - 2rem) .5rem, calc(100% - 2rem) -2%, 102% -2%, 102% 102%, -2% 102%) !important;
}

.user-input:focus {
  padding-inline-start: 0.9375rem !important;
}

.user-input-box label, .user-label {
  margin-left: -0.5rem !important;
}
</style>
