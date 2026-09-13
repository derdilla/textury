<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { createProject } from "@/services/api/managementApi";
import ModalPopover from "@/components/general/ModalPopover.vue";

defineExpose({ clear, submit });

const open = defineModel<boolean>("open", { default: false });

const emit = defineEmits<{
  (e: "success"): void;
}>();

let idModified = false;
const title = ref("");
const projectId = ref("");
// There is already a project with this id (Conflict).
// Must match <code>[a-zA-Z0-9.-_~!$&'()*+,;=:@]*</code>
const projectIdErr = ref("");
const isEditingId = ref(false);
const idInputField = ref<HTMLInputElement | null>(null);
const titleInputField = ref<HTMLInputElement | null>(null);

const sanitizeId = (str: string) => str.replaceAll(" ", "_").replaceAll(/([^a-zA-Z0-9.-_~!$&'()*+,;=:@])/g, "").toLowerCase();

function clear() {
  idModified = false;
  isEditingId.value = false;
  projectIdErr.value = "";
  title.value = "";
  projectId.value = "";
}

watch(open, async (isOpen) => {
  if (isOpen) {
    clear();
    await nextTick();
    titleInputField.value?.focus();
  }
});

watch(title, (newTitle) => {
  if (!idModified) {
    projectId.value = sanitizeId(newTitle);
  }
});

function handleProjectIdInput(event: Event) {
  const input = event.target as HTMLInputElement;
  const selectionStart = input.selectionStart ?? 0;
  const originalValue = input.value;

  const cleaned = sanitizeId(originalValue);
  const cursorLength = sanitizeId(originalValue.substring(0, selectionStart)).length;

  input.value = cleaned;
  projectId.value = cleaned;
  input.setSelectionRange(cursorLength, cursorLength);
}

async function startEditingId() {
  if (isEditingId.value) return;
  isEditingId.value = true;
  idModified = true;
  await nextTick();
  idInputField.value?.focus();
}

function finishEditingId() {
  isEditingId.value = false;
  if (!projectId.value.trim()) {
    idModified = false;
    projectId.value = sanitizeId(title.value);
  }
}

async function submit() {
  projectIdErr.value = "";
  const result = await createProject(projectId.value, title.value);
  if (result.success) {
    clear();
    open.value = false;
    emit("success");
  } else {
    projectIdErr.value = result.message;
  }
  return result.success;
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
      <div class="field small border no-margin no-round input-field" :class="{ 'invalid': projectIdErr }">
        <input ref="titleInputField" v-model="title" type="text" placeholder="Project title...">
      </div>
      <output v-if="projectIdErr" class="invalid error-text small-text small-padding">{{ projectIdErr }}</output>
    </div>

    <div class="modal-footer-actions">
      <div class="footer-left-info">
        <div class="id-chip" :class="{ editing: isEditingId }" @click="startEditingId">
          <template v-if="!isEditingId">
            <span class="grey-text small-text id-display">
              ID: <span class="id-text">{{ projectId || 'auto-generated' }}</span>
              <i class="edit-icon">edit</i>
            </span>
          </template>
          <template v-else>
            <span class="grey-text small-text id-label">ID:</span>
            <input ref="idInputField" class="minimal-input small-text grey-text" :value="projectId" @input="handleProjectIdInput"
              @blur="finishEditingId" @keydown.enter.prevent="finishEditingId" @click.stop type="text" placeholder="project-id">
          </template>
        </div>
      </div>
      <div class="footer-right-actions">
        <button tabindex="0" @click="cancel" class="border no-round">Cancel</button>
        <button tabindex="0" @click="submit" class="primary no-round" :disabled="!title.trim()" :class="{ 'is-disabled': !title.trim() }">
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
  align-items: center;
  margin-top: 1.5rem;
  width: 100%;
  gap: 1rem;
}

.footer-left-info {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  border-radius: 0px;
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