<script setup lang="ts">
import { ref, watch, nextTick, computed } from "vue";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";

const open = defineModel<boolean>("open", { default: false });
const entityName = defineModel<string>("fileName", { default: "" });

const props = defineProps<{
  kind: number | null;
}>();

const emit = defineEmits(["submit"]);

const errorMessage = ref("");
const inputField = ref<HTMLInputElement | null>(null);

enum DialogKind {
  createText = 1,
  createDir = 2,
}

const modalTitle = computed(() => {
  return props.kind === DialogKind.createDir ? "Add Folder" : "Add File";
});

const placeholderText = computed(() => {
  return props.kind === DialogKind.createDir ? "" : "main.tex";
});

function clear() {
  errorMessage.value = "";
  entityName.value = "";
}

watch(open, async (isOpen) => {
  if (isOpen) {
    clear();
    await nextTick();
    inputField.value?.focus();
  }
});

function handleFormSubmit() {
  if (!entityName.value.trim()) return;
  emit("submit");
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
        <h5 class="bold dialog-title no-margin">{{ modalTitle }}</h5>
      </div>
    </template>

    <div class="modal-body-content" @keydown.enter.prevent="handleFormSubmit">
      <div class="field label border no-round small no-margin input-field title-box">
        <input
          ref="inputField"
          v-model="entityName"
          type="text"
          class="title-input custom-placeholder"
          :placeholder="placeholderText"
        >
        <label class="title-label">Name</label>
      </div>
    </div>

    <template #actions>
      <div class="row gap">
        <button class="border no-round" tabindex="0" @click="cancel">Cancel</button>
        <button class="primary no-round" tabindex="0" @click="handleFormSubmit" :disabled="!entityName.trim()" :class="{ 'is-disabled': !entityName.trim() }">
          <i>done</i> Save
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

.is-disabled,
button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  pointer-events: none;
}

.title-input {
  padding-inline-start: 1rem !important;
  padding-inline-end: 1rem !important;
}

.title-input:is(:focus,[placeholder]:not(:placeholder-shown),.active) {
  clip-path: polygon(-2% -2%, 0.75rem -2%, 0.75rem .5rem, calc(100% - 2rem) .5rem, calc(100% - 2rem) -2%, 102% -2%, 102% 102%, -2% 102%) !important;
}

.title-input:focus {
  padding-inline-start: 0.9375rem !important;
}

.title-box label {
  margin-left: -0.5rem !important;
}

.title-label {
  margin-left: -0.5rem !important;
}

.custom-placeholder::placeholder {
  opacity: 0 !important;
  transition: opacity 0.2s ease;
}

.custom-placeholder:focus::placeholder {
  opacity: 1 !important;
}
</style>
