<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from "vue";
import ModalPopover from "@/components/ModalPopover.vue";

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    projectName?: string;
    projectTags: string[];
    availableTags?: string[];
  }>(),
  {
    projectName: "",
    availableTags: () => [],
  }
);

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  "update:tags": [tags: string[]];
  "create-tag": [tag: string];
}>();

const newTagInput = ref("");
const customTags = ref<string[]>([]);
const localTags = ref<string[]>([]);
const tagInputField = ref<HTMLInputElement | null>(null);

function handleGlobalKeyDown(event: KeyboardEvent) {
  if (event.key === "Enter" && document.activeElement !== tagInputField.value) {
    event.preventDefault();
    handleClose(false);
  }
}

// Watch modal visibility to initialize local tag state, auto-focus on open, clear input/emit changes on close and attach/remove global key listener.
watch(
  () => props.modelValue,
  async (isOpen, wasOpen) => {
    if (isOpen) {
      localTags.value = [...props.projectTags];
      customTags.value = [];
      newTagInput.value = "";
      window.addEventListener("keydown", handleGlobalKeyDown);
      await nextTick();
      tagInputField.value?.focus();
    } else {
      window.removeEventListener("keydown", handleGlobalKeyDown);
      if (wasOpen) {
        emitTagsIfChanged();
      }
    }
  }
);

onUnmounted(() => {
  window.removeEventListener("keydown", handleGlobalKeyDown);
});

function emitTagsIfChanged() {
  const currentSet = new Set(props.projectTags);
  const localSet = new Set(localTags.value);

  const isDifferent =
    currentSet.size !== localSet.size ||
    [...localSet].some((tag) => !currentSet.has(tag));

  if (isDifferent) {
    emit("update:tags", [...localTags.value]);
  }
}

function handleClose(value: boolean) {
  if (!value) {
    emitTagsIfChanged();
  }
  emit("update:modelValue", value);
}

const isSelected = (tag: string) => localTags.value.includes(tag);

// Combine all tag lists.
const allTags = computed(() => {
  const combined = [...props.availableTags, ...localTags.value, ...customTags.value];
  return Array.from(new Set(combined)).sort((a, b) =>
    a.localeCompare(b, undefined, { sensitivity: "base" })
  );
});

const trimmedInput = computed(() => newTagInput.value.trim());

// Disable add button when no input is available or tag is already selected locally.
const isAddDisabled = computed(() => {
  const input = trimmedInput.value.toLowerCase();
  return !input || localTags.value.some((t) => t.toLowerCase() === input);
});

function toggleTag(tag: string) {
  const active = isSelected(tag);

  if (active) {
    localTags.value = localTags.value.filter((t) => t !== tag);
    customTags.value = customTags.value.filter((t) => t !== tag);
  } else {
    localTags.value.push(tag);
  }
}

function handleCreateTag() {
  if (isAddDisabled.value) return;

  const input = trimmedInput.value;
  // Check if tag already exists.
  const match = allTags.value.find((t) => t.toLowerCase() === input.toLowerCase());
  const tagToApply = match || input;

  // If tag is new, send event.
  if (!match) {
    customTags.value.push(tagToApply);
    emit("create-tag", tagToApply);
  }

  if (!isSelected(tagToApply)) {
    localTags.value.push(tagToApply);
  }

  newTagInput.value = "";
}

function handleInputEnter() {
  if (trimmedInput.value) {
    handleCreateTag();
  } else {
    handleClose(false);
  }
}
</script>

<template>
  <ModalPopover :model-value="modelValue" @update:model-value="handleClose" max-width="500px">
    <template #header>
      <div class="max dialog-header-content">
        <h5 class="bold dialog-title no-margin">Tags</h5>
        <p v-if="projectName" class="grey-text no-margin project-subtitle" :title="projectName">{{ projectName }}</p>
      </div>
    </template>

    <div class="tags-modal-body">
      <div class="create-tag-section">
        <div class="field small border no-margin no-round input-field">
          <input ref="tagInputField" v-model="newTagInput" type="text" placeholder="Create new tag..." @keydown.enter.stop.prevent="handleInputEnter">
        </div>
        <button tabindex="0" class="square secondary-container no-round add-button" :class="{ 'is-disabled': isAddDisabled }" :disabled="isAddDisabled" @click="handleCreateTag">
          <i>add</i>
        </button>
      </div>

      <div v-if="allTags.length > 0" class="tags-wrap-grid">
        <button v-for="tag in allTags" :key="tag" tabindex="0" @click="toggleTag(tag)" class="chip medium no-border no-round"
          :class="{ 'primary': isSelected(tag), 'surface-container-highest': !isSelected(tag) }">
          <i>{{ isSelected(tag) ? 'check' : 'add' }}</i>
          <span>{{ tag }}</span>
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

.project-subtitle {
  padding-top: 0.3rem;
  font-size: 0.9rem;
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
  padding-bottom: 0.5rem;
}

.tags-modal-body {
  display: flex;
  flex-direction: column;
}

.create-tag-section {
  display: flex;
  align-items: center;
  gap: 1rem;
  width: 100%;
}

.input-field {
  flex: 1;
  width: 100%;
}

.add-button {
  flex-shrink: 0;
  height: 2.5rem;
  width: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: opacity 0.2s ease, cursor 0.2s ease;
}

.add-button.is-disabled,
.add-button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  pointer-events: none;
}

.tags-wrap-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  border-radius: 0 !important;
  margin-top: 1.5rem;
}
</style>