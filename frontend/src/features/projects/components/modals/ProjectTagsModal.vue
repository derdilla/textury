<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from "vue";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import { useProjectsQuery } from "@/features/projects/composables/projectsQuery";

const open = defineModel<boolean>("open", { default: false });

const props = defineProps<{
  projectId: string;
}>();

const projectsStore = useProjectsStore();
const projectsQuery = useProjectsQuery();

const project = computed(() => projectsStore.getProjectById(props.projectId));
const projectTitle = computed(() => project.value?.title ?? "");
const storeProjectTags = computed(() => project.value?.metadata?.tags ?? []);

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

watch(
  open,
  async (isOpen, wasOpen) => {
    if (isOpen) {
      localTags.value = [...storeProjectTags.value];
      customTags.value = [];
      newTagInput.value = "";
      window.addEventListener("keydown", handleGlobalKeyDown);
      await nextTick();
      tagInputField.value?.focus();
    } else {
      window.removeEventListener("keydown", handleGlobalKeyDown);
      if (wasOpen) {
        await saveTagsIfChanged();
      }
    }
  }
);

onUnmounted(() => {
  window.removeEventListener("keydown", handleGlobalKeyDown);
});

async function saveTagsIfChanged() {
  if (!project.value) return;

  const currentSet = new Set(storeProjectTags.value);
  const localSet = new Set(localTags.value);

  const isDifferent =
    currentSet.size !== localSet.size ||
    [...localSet].some((tag) => !currentSet.has(tag));

  if (isDifferent) {
    await projectsStore.updateMetadata(props.projectId, {
      ...project.value.metadata,
      tags: [...localTags.value],
    });
  }
}

function handleClose(value: boolean) {
  open.value = value;
}

const isSelected = (tag: string) => localTags.value.includes(tag);

const allTags = computed(() => {
  const combined = [...projectsQuery.availableTags.value, ...localTags.value, ...customTags.value];
  return Array.from(new Set(combined)).sort((a, b) =>
    a.localeCompare(b, undefined, { sensitivity: "base" })
  );
});

const trimmedInput = computed(() => newTagInput.value.trim());

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
  const match = allTags.value.find((t) => t.toLowerCase() === input.toLowerCase());
  const tagToApply = match || input;

  if (!match) {
    customTags.value.push(tagToApply);
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
  <ModalPopover v-model:open="open" max-width="500px">
    <template #header>
      <div class="max dialog-header-content">
        <h5 class="bold dialog-title no-margin">Tags</h5>
        <p v-if="projectTitle" class="grey-text no-margin project-subtitle" :title="projectTitle">{{ projectTitle }}</p>
      </div>
    </template>

    <div class="tags-modal-body">
      <div class="create-tag-section">
        <div class="field label small border no-margin no-round input-field tag-box">
          <input ref="tagInputField" v-model="newTagInput" type="text" @keydown.enter.stop.prevent="handleInputEnter" class="tag-input">
          <label class="tag-label">New tag</label>
        </div>
        <FloatingElement hover align="left">
          <template #button>
            <button
              tabindex="0"
              class="square secondary-container no-round add-button"
              :class="{ 'is-disabled': isAddDisabled }"
              :disabled="isAddDisabled"
              @click="handleCreateTag"
            >
              <i>add</i>
            </button>
          </template>
          <template #menu>
            <span class="tooltip bottom active">Add Tag</span>
          </template>
        </FloatingElement>
      </div>

      <div v-if="allTags.length > 0" class="tags-wrap-grid">
        <button
          v-for="tag in allTags"
          :key="tag"
          tabindex="0"
          @click="toggleTag(tag)"
          class="chip medium no-border no-round"
          :class="{ 'primary': isSelected(tag), 'surface-container-highest': !isSelected(tag) }"
        >
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

.tag-input {
  /* Overwrite default BeerCSS paddings that come with prefix and suffix. */
  padding-inline-start: 1rem !important;
  padding-inline-end: 1rem !important;
}
.tag-input:is(:focus,[placeholder]:not(:placeholder-shown),.active) {
  /* Slightly changed mask from BeerCSS. Changed 1.5rem to 0.75rem. */
  clip-path: polygon(-2% -2%, 0.75rem -2%, 0.75rem .5rem, calc(100% - 2rem) .5rem, calc(100% - 2rem) -2%, 102% -2%, 102% 102%, -2% 102%) !important;
}
.tag-input:focus {
  /* Offset of 0.0625 is needed to stop the text from jumping when focusing. Derived from BeerCSS. */
  padding-inline-start: 0.9375rem !important;
}
.tag-box label {
  margin-left: -0.5rem !important;
}
.tag-label {
  margin-left: -0.5rem !important;
}
</style>
