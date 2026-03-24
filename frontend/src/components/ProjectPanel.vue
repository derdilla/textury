<script setup lang="ts">
import { ProjectApi, type FileMetadata } from '@/services/projectApi';
import { inject, ref, watch } from 'vue';
import FileTree from './FileTree.vue';
import type { EditorController } from '@/services/editorControllerType';
import {editCtrl} from "@/services/editorController.ts";

const addDialog = ref()
const fileName = defineModel('fileName')

const dirIndex = ref<FileMetadata | null>(null);
async function updateIndex(api: ProjectApi) {
  const index = await api?.index();
  if (index) dirIndex.value = index;
}

const api = ref<ProjectApi | null>(null);
if (editCtrl?.openProject) {
  api.value = new ProjectApi(editCtrl.openProject);
}

watch(
  () => editCtrl?.openProject,
  async (newProject) => {
    if (!newProject) return;
    dirIndex.value = null;
    api.value = new ProjectApi(newProject);
    await updateIndex(api.value);
  },
);

function openDialog() {
  addDialog.value?.show();
  fileName.value = '';
}

function closeDialog() {
  addDialog.value?.close();
}

async function addFile() {
  if (api.value == null) return;
  if (fileName.value) {
    await api.value.createOrUpdateFile(fileName.value, null);
  }
  closeDialog();
  await updateIndex(api.value);
}

if (api.value != null) {
  updateIndex(api.value);
}
</script>

<template>
  <nav class="left panel round width-limited">
    <button @click="editCtrl!.sidebarHidden != editCtrl.sidebarHidden" class="transparent circle extra small-margin">
      <i>menu</i>
    </button>
    <h5 class="small no-padding no-margin">Project Title</h5>
    <div v-if="dirIndex !== null" id="tree-container">
      <FileTree v-for="child in dirIndex.file_type.Directory.children" v-bind:key="child.name" :root="child" />
    </div>
    <i v-if="dirIndex == null">upload_file</i>
    <p v-if="dirIndex == null" class="small-text">No files yet</p>
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
  overflow: clip;
}

#tree-container {
  width: 100%;
  display: flex;
  flex-direction: column;
  padding-left: 0.7em;
  align-items: start;
  border-radius: 0;
}
</style>
