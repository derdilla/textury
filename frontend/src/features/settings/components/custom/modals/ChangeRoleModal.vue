<script setup lang="ts">
import { ref, watch } from "vue";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";
import type { PrivateAccountInfo } from "@/api/types";

const open = defineModel<boolean>("open", { default: false });
const props = defineProps<{
  user: PrivateAccountInfo | null;
}>();

const emit = defineEmits<{
  (e: "save", isAdmin: boolean): void;
}>();

const isAdmin = ref(false);

watch(open, (isOpen) => {
  if (isOpen && props.user) {
    isAdmin.value = props.user.isAdmin;
  }
}, { immediate: true });

function submit() {
  emit("save", isAdmin.value);
  open.value = false;
}
</script>

<template>
  <ModalPopover v-model:open="open" max-width="500px">
    <template #header>
      <div class="max dialog-header-content">
        <h5 class="bold dialog-title no-margin">Change Role</h5>
      </div>
    </template>

    <div class="modal-body-content">
      <div class="field label border suffix small">
        <select v-model="isAdmin" class="no-round active">
          <option :value="true">Admin</option>
          <option :value="false">User</option>
        </select>
        <label class="active">Role</label>
        <i class="select-icon">keyboard_arrow_down</i>
      </div>
    </div>

    <template #actions>
      <div class="row gap">
        <button class="border no-round" @click="open = false">Cancel</button>
        <button class="primary no-round" @click="submit">
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
.select-icon {
  transform: translate(5px, 0px);
}
</style>
