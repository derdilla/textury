<script setup lang="ts">
import { ProjectApi, type FileMetadata } from '@/services/projectApi';
import { ref, watch } from 'vue';
import FileTree from './FileTree.vue';
import { editCtrl, updateIndex } from "@/services/editorController.ts";

const addDialog = ref<HTMLDialogElement>();
const fileName = defineModel<string | null>('fileName');

function openDialog() {
  addDialog.value?.show();
  fileName.value = '';
}

function closeDialog() {
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
  <nav class="left panel width-limited">
    <div id="title">
      <button @click="editCtrl!.sidebarHidden = !editCtrl.sidebarHidden" class="transparent circle no-margin">
        <i>menu</i>
      </button>
      <h5 class="small no-padding no-margin">
        <a href="/projects">{{ editCtrl.index?.summary.title ?? editCtrl.openProject }}</a>
      </h5>
    </div>

    <div v-if="editCtrl.dirIndex !== null" id="tree-container">
      <FileTree v-for="child in editCtrl.dirIndex.file_type.Directory.children" v-bind:key="child.name" :root="child" />
    </div>
    <i v-if="editCtrl.dirIndex == null">upload_file</i>
    <p v-if="editCtrl.dirIndex == null" class="small-text">No files yet</p>
    <div class="max"></div>
    <dialog class="responsive" ref="addDialog">
      <nav>
        <button @click="closeDialog" class="circle transparent"><i>close</i></button>
        <h6 class="max">Add File</h6>

        <button @click="addFile" class="outline"><i>done</i> <span>Save</span></button>
      </nav>
      <div class="field border">
        <input v-model="fileName" type="text" placeholder="foo/bar/file.txt">
      </div>
    </dialog>
    <button @click="openDialog" class="border small-margin"><i>add</i> Add</button>
  </nav>
</template>

// TODO: figure out how to style nested lists nicely
<style scoped>

.width-limited * {
  max-width: 15vw;
  overflow-x: scroll;
}

.width-limited dialog * {
  max-width: none;
}

.width-limited *:has(> menu) {
  overflow: visible;
}

#title {
  align-self: start;
  display: ruby;
}

menu {
  width: auto;
}

#tree-container {
  width: 100%;
  flex-grow: 9;
  display: flex;
  flex-direction: column;
  padding-left: 0.7em;
  align-items: start;
  justify-content: start;
}

#tree-container > * {
  width: inherit;
}

</style>
