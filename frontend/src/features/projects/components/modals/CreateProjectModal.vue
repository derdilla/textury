<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import { useProjectsLayoutStore, type ProjectTemplateOption } from "@/features/projects/stores/projectsLayoutStore";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";

const open = defineModel<boolean>("open", { default: false });

const projectsStore = useProjectsStore();
const projectsLayout = useProjectsLayoutStore();

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
  const result = await projectsStore.createProject(title.value, selectedTemplate.value);

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

const templateOptions: { label: string; value: ProjectTemplateOption }[] = [
  { label: "None", value: "none" },
  { label: "LaTeX", value: "latex" },
  { label: "Typst", value: "typst" },
];

const selectedTemplate = ref<ProjectTemplateOption>(projectsLayout.getLastUsedTemplate());

function onTemplateChange(e: Event) {
  const value = (e.target as HTMLSelectElement).value as ProjectTemplateOption;
  selectedTemplate.value = value;
  projectsLayout.setLastUsedTemplate(value);
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
      <div v-if="errorMessage" class="chip max no-round error-text grid bottom-margin error-box">
        <i class="s1">error</i>
        <span class="s11 wrap text-wrap">{{ errorMessage }}</span>
      </div>
      <div class="field label border no-round small no-margin input-field" :class="{ 'title-box': true, 'invalid': errorMessage }">
        <input ref="titleInputField" v-model="title" type="text" class="title-input">
        <label class="title-label">Title</label>
      </div>
      <div class="field label border suffix small">
        <select :value="selectedTemplate" class="no-round" @change="onTemplateChange">
          <option v-for="opt in templateOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
        <label>Template</label>
        <i class="select-icon">keyboard_arrow_down</i>
      </div>
    </div>

    <template #actions>
      <div class="row gap">
        <button class="border no-round" tabindex="0" @click="cancel">Cancel</button>
        <button class="primary no-round" tabindex="0" @click="submit" :disabled="!title.trim()" :class="{ 'is-disabled': !title.trim() }">
          <i>arrow_forward</i> Create
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
  /* Overwrite default BeerCSS paddings that come with prefix and suffix. */
  padding-inline-start: 1rem !important;
  padding-inline-end: 1rem !important;
}
.title-input:is(:focus,[placeholder]:not(:placeholder-shown),.active) {
  /* Slightly changed mask from BeerCSS. Changed 1.5rem to 0.75rem. */
  clip-path: polygon(-2% -2%, 0.75rem -2%, 0.75rem .5rem, calc(100% - 2rem) .5rem, calc(100% - 2rem) -2%, 102% -2%, 102% 102%, -2% 102%) !important;
}
.title-input:focus {
  /* Offset of 0.0625 is needed to stop the text from jumping when focusing. Derived from BeerCSS. */
  padding-inline-start: 0.9375rem !important;
}
.title-box label {
  margin-left: -0.5rem !important;
}
.title-label {
  margin-left: -0.5rem !important;
}
.select-icon {
  transform: translate(5px, 0px);
}

.error-box {
  overflow: hidden;
  padding-top: 0.6rem;
  padding-bottom: 0.6rem;
}
</style>
