<script setup lang="ts">
import { ProjectApi, type FileMetadata } from '@/services/projectApi';
import { nextTick, ref, watch } from 'vue';
import FileTree from './FileTree.vue';
import { editCtrl, updateIndex } from "@/services/editorController.ts";
import router from "@/router";

const addDialog = ref<HTMLDialogElement>();
const fileName = defineModel<string | null>('fileName');

const fileNameInput = ref<HTMLInputElement>();

function openDialog() {
  addDialog.value?.show();
  fileName.value = '';

  //Delay until rendering of the next frame so the focus actually works. No idea why this is necessary. 
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      fileNameInput.value?.focus();
    });
  });

  console.log(editCtrl.dirIndex == null)
}

function closeDialog() {
  fileNameInput.value?.blur();
  addDialog.value?.close();
}

async function addFile() {
  if (!editCtrl.openProject) return;
  const api = new ProjectApi(editCtrl.openProject);
  if (fileName.value) {
    await api.createOrUpdateFile(fileName.value, null);
  }
  closeDialog();
  await updateIndex(api);
}

async function backToHome() {
  await router.push('/projects');
}
</script>

<template>
  <div class="panel surface-container">
    <nav class="header-container top-left margin">
      <button @click="backToHome" class="square">
        <i>home</i>
      </button>
      <div class="title-stack max field">
        <span class="project-title truncate-text bold">{{ editCtrl.index?.summary.title ?? editCtrl.openProject }}</span>
        <span class="project-owner truncate-text">{{ editCtrl.index?.summary.owner.name ?? editCtrl.openProject }}</span>
      </div>
    </nav>
    <main class="main-container left-margin field">
      <div v-if="editCtrl.dirIndex !== null" class="tree-container">
        <FileTree v-for="child in editCtrl.dirIndex.file_type.Directory.children" v-bind:key="child.name" :root="child" />
      </div>
      <i v-if="editCtrl.dirIndex == null">upload_file</i>
      <p v-if="editCtrl.dirIndex == null" class="small-text">No files yet</p>
      <dialog class="responsive" ref="addDialog">
        <nav>
          <button @click="closeDialog" class="circle transparent"><i>close</i></button>
          <h6 class="max">Add File</h6>

          <button @click="addFile" class="outline"><i>done</i> <span>Save</span></button>
        </nav>
        <div class="field border">
          <input v-model="fileName" type="text" placeholder="foo/bar/file.tex" @keyup.enter="addFile" ref="fileNameInput">
        </div>
      </dialog>
      <button @click="openDialog" class="fill square absolute right bottom margin">
        <i>add</i>
      </button>
    </main>
  </div>
</template>

<style scoped>
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
  flex-grow: 1;
}

.tree-container {
  flex-grow: 1;
  overflow-x: hidden;
}

.truncate-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
