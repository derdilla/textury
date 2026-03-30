<script setup lang="ts">
import { ProjectApi, type FileMetadata } from '@/services/projectApi';
import { nextTick, ref, watch } from 'vue';
import FileTree from './FileTree.vue';
import { editCtrl, updateIndex } from "@/services/editorController.ts";
import { log } from 'console';

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
</script>

<template>
  <div class="panel">
    <div class="header-container">
      <a href="/projects" class="home-button"></a>
      <div class="title-stack">
        <span class="project-title">{{ editCtrl.index?.summary.title ?? editCtrl.openProject }}</span>
        <span class="project-owner">{{ editCtrl.index?.summary.owner.name ?? editCtrl.openProject }}</span>
      </div>
    </div>
    <div class="main-container">
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
      <button @click="openDialog" class="add-button"></button>
    </div>
  </div>
</template>

<style scoped>

.panel {
  background-color: #1f2128;
  height: 100vh;
  display: flex;
  flex-direction: column;
  flex-wrap: nowrap;
  justify-content: flex-start;
  align-items: stretch;
  overflow: hidden;
  border-top-right-radius: 25px;
  border-bottom-right-radius: 25px;
}

.header-container {
  padding: 15px;
  display: flex;
  flex-wrap: nowrap;
  justify-content: flex-start;
  align-items: center;
  gap: 15px;
  flex-shrink: 0;
}

.home-button {
  background-color: #57a2b5;
  border-radius: 12px;
  width: 45px;
  height: 45px;
  background-image: url("../assets/icons/home.svg");
  background-repeat: no-repeat;
  flex-shrink: 0;
}

.home-button:hover {
  background-color: #57a2b5d3;
  background-blend-mode: luminosity;
}

.title-stack {
  display: flex;
  flex-direction: column;
  justify-content: center;
  flex-grow: 1;
  overflow: hidden;
  height: 45px;
  padding-top: 3px;
}

.project-title {
  font-family: Avenir, Montserrat, Corbel, 'URW Gothic', source-sans-pro, sans-serif;
  font-weight: bold;
  font-size: 20px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.project-owner {
  font-size: small;
  color: #a0a0a0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.main-container {
  position: relative;
  flex-grow: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.tree-container {
  flex-grow: 1;
  overflow-y: auto;
  overflow-x: ;
  padding: 15px;

}

.add-button {
  position: absolute;
  padding: 0;
  bottom: 15px;
  right: 15px;
  background-color: #57a2b5;
  border-radius: 12px;
  width: 45px !important;
  height: 45px;
  background-image: url("../assets/icons/add.svg");
  background-repeat: no-repeat;
  z-index: 999;
}

.add-button:hover {
  background-color: #57a2b5d3;
  background-blend-mode: luminosity;
}

</style>
