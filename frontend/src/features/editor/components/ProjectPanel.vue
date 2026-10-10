<script setup lang="ts">
import { ref, computed } from 'vue';
import FileTree from '@/features/editor/components/FileTree.vue';
import router from "@/router";
import { useContactsStore } from '@/shared/stores/contactsStore';
import { useEditorSession } from "@/features/editor/composables/editorSession";
import { useFileDrop } from '@/features/editor/composables/fileDragNDrop';
import AddEntityModal from '@/features/editor/components/modals/AddEntityModal.vue';

const editorSession = useEditorSession();
const contactsStore = useContactsStore();

const addDialog = ref<HTMLDialogElement>();
const fileName = defineModel<string | null>('fileName');

const isAddModalOpen = ref(false);
const entityName = ref<string>("");

enum DialogKind {
  createText = 1,
  createDir = 2,
}

let lastOpenedDialogKind = ref<DialogKind | null>(null);

function openDialog(kind: DialogKind) {
  console.info(`openDialog(${kind})`);
  entityName.value = '';
  lastOpenedDialogKind.value = kind;
  isAddModalOpen.value = true;
}

function closeDialog() {
  isAddModalOpen.value = false;
}

async function addFile() {
  if (!lastOpenedDialogKind.value) {
    console.warn('Missing lastOpenedDialogKind');
    closeDialog();
    return;
  }
  if (!editorSession.projectId.value) {
    console.warn('Missing editorSession.projectId');
    closeDialog();
    return;
  }
  if (!entityName.value) {
    console.warn('Missing fileName');
    closeDialog();
    return;
  }

  let res: string | undefined = 'invalidKind';
  switch (lastOpenedDialogKind.value) {
    case DialogKind.createText:
      res = editorSession.currentTree.value?.root.value?.createTextFile(entityName.value);
      break;
    case DialogKind.createDir:
      res = editorSession.currentTree.value?.root.value?.createDir(entityName.value);
      break;
  }
  if (res !== 'ok') {
    console.error(res);
    return;
  }
  closeDialog();
}

async function backToHome() {
  await router.push('/projects');
}

const fileDrop = computed(() => {
  const root = editorSession.currentTree.value?.root.value;
  if (!root) return null;
  return useFileDrop(root);
});

function handleDrop(e: DragEvent) {
  if (!editorSession.projectId.value) {
    fileDrop.value?.onDrop(e, null);
  } else {
    fileDrop.value?.onDrop(e, editorSession.currentTree.value);
  }
}

const sortedChildren = computed(() => editorSession.currentTree.value?.root.value?.children.toSorted((a, b) => {
  if (a.kind === 'directory' && b.kind !== 'directory') return -1;
  if (a.kind !== 'directory' && b.kind === 'directory') return 1;
  if (a.name < b.name) return -1;
  if (a.name > b.name) return 1;
  return 0;
}));

const emit = defineEmits(['openProjectSettings']);

</script>

<template>
  <div class="panel surface-container">
    <nav class="header-container top-left margin">
      <button @click="backToHome" class="square">
        <i>home</i>
      </button>
      <div class="title-stack max field">
        <span class="project-title truncate-text bold">{{ editorSession.projectSummary.value?.title ?? "" }}</span>
        <span class="project-owner truncate-text">{{ editorSession.projectSummary.value ? contactsStore.getUsername(editorSession.projectSummary.value.owner) : "" }}</span>
      </div>
    </nav>
    <main class="main-container no-margin">
      <div v-if="editorSession.currentTree.value?.root.value" class="tree-container"
        :class="{ 'global-drag-active': fileDrop?.isDragOver.value }" @dragover.prevent
        @dragenter="fileDrop?.onEnter" @dragleave="fileDrop?.onLeave" @drop="handleDrop">
        <FileTree v-for="child in sortedChildren" :key="child.name" :root="child" />
      </div>
      <div v-show="editorSession.currentTree.value?.root.value == null" class="center-align middle">
        <i class="extra">upload_file</i>
        <p class="h5">No files yet.</p>
      </div>

      <AddEntityModal
        v-model:open="isAddModalOpen"
        v-model:fileName="entityName"
        :kind="lastOpenedDialogKind"
        @submit="addFile"
      />

      <div class="row bottom-bar margin no-padding">
        <button @click="emit('openProjectSettings')" class="transparent square surface-container-high">
          <i>settings</i>
        </button>
        <div class="max"></div>
        <button @click="openDialog(DialogKind.createDir)" class="fill square ">
          <i>create_new_folder</i>
        </button>
        <button @click="openDialog(DialogKind.createText)" class="fill square">
          <i>note_add</i>
        </button>
      </div>
    </main>
  </div>
</template>

<style scoped>
.global-drag-active {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 99%);
  outline: 2px dashed color-mix(in srgb, var(--inverse-surface), transparent 80%);
  outline-offset: -4px;
  border-radius: 8px;
}

.panel {
  height: 100vh;
  display: flex;
  flex-direction: column;
  border-radius: 0 18px 18px 0;
}

.panel > * {
  border-radius: 0;
}

.header-container {
  height: 2.5rem;
}

.title-stack {
  overflow: hidden;
}

.project-title {
  font-size: 20px;
  margin-top: 5px;
}

.project-owner {
  font-size: small;
  opacity: 0.5;
}

.main-container {
  display: flex;
  flex-direction: column;
  flex-grow: 1;
}

.tree-container {
  flex-grow: 1;
  overflow-x: hidden;
  padding-left: 8.5px;
  padding-right: 8.5px;
  padding-bottom: 0.5rem;
  padding-top: 0.5rem;
}

.bottom-bar {
  margin-top: auto;
  flex-shrink: 0;
}

.truncate-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
