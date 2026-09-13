<script setup lang="ts">
import { ref, computed } from 'vue';
import FileTree from './FileTree.vue';
import { editCtrl } from "@/services/editorController.ts";
import router from "@/router";

const addDialog = ref<HTMLDialogElement>();
const fileName = defineModel<string | null>('fileName');

const fileNameInput = ref<HTMLInputElement>();

function openDialog() {
  addDialog.value?.show();
  fileName.value = '/';

  //Delay until rendering of the next frame so the focus actually works. No idea why this is necessary.
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      fileNameInput.value?.focus();
    });
  });

}

function closeDialog() {
  fileNameInput.value?.blur();
  addDialog.value?.close();
}

async function addFile() {
  if (!editCtrl.openProject) return;
  //const api = new ProjectApi(editCtrl.openProject);
  if (fileName.value) {
    const err = editCtrl.tree.createTextFile(fileName.value);
    if (err) {
      console.error(err);
      return;
    }
    //await api.createOrUpdateFile(fileName.value, null);
  }
  closeDialog();
  //await updateIndex(api);
}

async function backToHome() {
  await router.push('/projects');
}

const globalDragCounter = ref(0);
const showGlobalHighlight = ref(false);
let delayTimer: null | number = null;

const isGlobalDragOver = computed(() => globalDragCounter.value > 0 && showGlobalHighlight.value);

function handleGlobalEnter(e: DragEvent) {
  e.preventDefault();
  globalDragCounter.value++;
  // Added a slight delay to stop flickering of the highlight when dragging between files.
  if (!delayTimer) {
    delayTimer = setTimeout(() => {
      showGlobalHighlight.value = true;
    }, 400);
  }
}

function handleGlobalLeave() {
  globalDragCounter.value = Math.max(0, globalDragCounter.value - 1);
  if (globalDragCounter.value === 0) {
    if (delayTimer) {
      clearTimeout(delayTimer);
      delayTimer = null;
    }
    showGlobalHighlight.value = false;
  }
}

function onGlobalDrop(e: DragEvent) {
  if (delayTimer) {
    clearTimeout(delayTimer);
    delayTimer = null;
  }
  globalDragCounter.value = 0;
  showGlobalHighlight.value = false;
  handleGlobalDrop(e);
}

async function handleGlobalDrop(e: DragEvent) {
  e.preventDefault();
  if (!e.dataTransfer || !editCtrl.openProject) return;

  // SCENARIO: File/Directory Move
  const sourcePath = e.dataTransfer.getData('application/file-path');

  if (sourcePath) {
    const fileName = sourcePath.split('/').filter(Boolean).pop();
    const destinationPath = '/' + fileName;

    if (sourcePath !== destinationPath) {
      // Execute file moving.
      await handlePathChange(sourcePath, destinationPath);
    }
    return;
  }

  // SCENARIO: File Upload
  const fileItems = [...e.dataTransfer.items]
    .map((item) => item.getAsFile())
    .filter((file) => file !== null) as File[];

  if (fileItems.length > 0) {
    for (const file of fileItems) {
      const uploadPath = '/' + file.name;

      // Execute file uploading.
      let err: string | null;
      if (file.type.startsWith('text/')) {
        err = editCtrl.tree.createTextFile(uploadPath, await file.text());
      } else {
        err = editCtrl.tree.uploadAsset(uploadPath, await file.bytes());
      }
      if (err) console.error(`Upload error: ${err}`);
    }
  }
}

function handlePathChange(oldPath: string, newPath: string): boolean {
  // effectively handleMove
  if (!editCtrl.openProject) return false;

  const res = editCtrl.tree.moveEntry(oldPath, newPath); // Execute file moving.

  if (res) {
    console.log(res);
    return false;
  }

  return true;
}
</script>

<template>
  <div class="panel surface-container">
    <nav class="header-container top-left margin">
      <button @click="backToHome" class="square">
        <i>home</i>
      </button>
      <div class="title-stack max field">
        <span class="project-title truncate-text bold">{{ editCtrl.index?.summary.title ?? editCtrl.openProject
          }}</span>
        <span class="project-owner truncate-text">{{ editCtrl.index?.summary.owner.name ?? editCtrl.openProject
          }}</span>
      </div>
    </nav>
    <main class="main-container field no-margin">
      <div v-if="editCtrl.tree.dirIndex !== null" class="tree-container"
        :class="{ 'global-drag-active': isGlobalDragOver }" @dragover.prevent @dragenter="handleGlobalEnter"
        @dragleave="handleGlobalLeave" @drop="onGlobalDrop">
        <FileTree v-for="child in editCtrl.tree.dirIndex.file_type.Directory?.children ?? []" v-bind:key="child.name"
          :root="child" :level="0" />
      </div>
      <div v-show="editCtrl.tree.dirIndex == null" class="center-align middle">
        <i class="extra">upload_file</i>
        <p class="h5">No files yet.</p>
      </div>
      <dialog class="responsive" ref="addDialog">
        <nav>
          <button @click="closeDialog" class="circle transparent"><i>close</i></button>
          <h6 class="max">Add File</h6>

          <button @click="addFile" class="outline"><i>done</i> <span>Save</span></button>
        </nav>
        <div class="field border">
          <input v-model="fileName" type="text" placeholder="foo/bar/file.tex" @keyup.enter="addFile"
            ref="fileNameInput">
        </div>
      </dialog>
      <button @click="openDialog" class="fill square absolute right bottom margin">
        <i>add</i>
      </button>
    </main>
  </div>
</template>

<style scoped>
.global-drag-active {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 99%);
  outline: 2px dashed color-mix(in srgb, var(--inverse-surface), transparent 80%);
  outline-offset: -4px;
}

.panel {
  height: 100vh;
  display: flex;
  flex-direction: column;
  border-radius: 0 18px 18px 0;
}

.panel>* {
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
  flex-grow: 1;
}

.tree-container {
  flex-grow: 1;
  overflow-x: hidden;
  padding-left: 8.5px;
  padding-right: 8.5px;
  padding-bottom: 4rem;
  padding-top: 1rem;
}

.truncate-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
