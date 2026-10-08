<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";

const open = defineModel<boolean>("open", { default: false });

const projectsStore = useProjectsStore();

const title = ref("");

const errorMessage = ref("");
const isEditingId = ref(false);

const titleInputField = ref<HTMLInputElement | null>(null);

function clear() {
  isEditingId.value = false;
  errorMessage.value = "";
  title.value = "";
}

watch(open, async (isOpen) => {
  if (isOpen) {
    clear();
    await nextTick();
    titleInputField.value?.focus();
  }
});


async function submit() {
  if (!title.value.trim()) return false;

  errorMessage.value = "";
  const result = await projectsStore.createProject(title.value);

  if (result.success) {
    clear();
    open.value = false;
    return true;
  } else {
    errorMessage.value = result.error || "An error occurred";
    return false;
  }
}

function cancel() {
  clear();
  open.value = false;
}
</script>

<template>
  <ModalPopover v-model:open="open" max-width="500px">
    <template #header>
      <div class="max dialog-header-content">
        <h5 class="bold dialog-title no-margin">Create Project</h5>
      </div>
    </template>

    <div class="modal-body-content" @keydown.enter.prevent="submit">
      <div v-if="errorMessage" class="chip no-round error-text row medium no-padding bottom-margin">
        <i class="large-padding left-padding">error</i>
        <span class="large-padding right-padding">{{ errorMessage }}</span>
      </div>
      <div class="field small border no-margin no-round input-field" :class="{ 'invalid': errorMessage }">
        <input ref="titleInputField" v-model="title" type="text" placeholder="Project title...">
      </div>
    </div>

    <div class="modal-footer-actions">
      <div style="flex: 1"></div>
      <div class="footer-right-actions row right bottom">
        <button tabindex="0" @click="cancel" class="border no-round">Cancel</button>
        <button
          tabindex="0"
          @click="submit"
          class="primary no-round"
          :disabled="!title.trim()"
          :class="{ 'is-disabled': !title.trim() }"
        >
          <i>done</i>
          <span>Create</span>
        </button>
      </div>
    </div>
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
  margin-bottom: 2rem;
}

.input-field {
  width: 100% !important;
}

.modal-footer-actions {
  display: flex;
  justify-content: space-between;
  align-items: right;
  margin-top: 1.5rem;
  width: 100%;
  gap: 1rem;
}

.footer-right-actions {
  display: flex;
  gap: 1rem;
  align-items: center;
  flex-shrink: 0;
}

.id-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.25rem 0.5rem;
  border-radius: 8px;
  background-color: transparent;
  cursor: pointer;
  transition: background-color 0.15s ease;
  max-width: 100%;
  overflow: hidden;
  height: 2.5rem;
}

.id-chip:hover {
  background-color: var(--surface-container-highest, rgba(255, 255, 255, 0.08));
}

.id-chip.editing {
  background-color: var(--surface-container-highest, rgba(255, 255, 255, 0.08));
  cursor: default;
  display: flex;
}

.id-display {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  min-width: 0;
  overflow: hidden;
}

.id-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.id-label {
  flex-shrink: 0;
}

.edit-icon {
  font-size: 0.85rem;
  opacity: 0;
  transition: opacity 0.15s ease;
  flex-shrink: 0;
}

.id-chip:hover .edit-icon {
  opacity: 0.8;
}

.minimal-input {
  background: transparent;
  border: none;
  outline: none;
  color: inherit;
  font-family: inherit;
  font-size: inherit;
  padding: 0;
  margin: 0;
  flex: 1;
  min-width: 0;
  width: 100%;
  border-radius: 0;
}

.is-disabled,
button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  pointer-events: none;
}
</style>
