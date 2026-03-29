<script setup lang="ts">
import { ProjectApi, type FileMetadata } from '@/services/projectApi';
import { ref, watch } from 'vue';
import FileTree from './FileTree.vue';
import { editCtrl } from "@/services/editorController.ts";

const addDialog = ref<HTMLDialogElement>();
const fileName = defineModel('fileName');

async function updateIndex(api: ProjectApi) {
  const index = await api?.index();
  if (index) editCtrl.dirIndex = index;
}

const api = ref<ProjectApi | null>(null);
if (editCtrl?.openProject) {
  api.value = new ProjectApi(editCtrl.openProject);
}

watch(
  () => editCtrl?.openProject,
  async (newProject) => {
    if (!newProject) return;
    editCtrl.dirIndex = null;
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
  <nav class="left panel width-limited">
    <div id="title">
      <button @click="editCtrl!.sidebarHidden = !editCtrl.sidebarHidden" class="transparent circle no-margin">
        <i>menu</i>
      </button>
      <h5 class="small no-padding no-margin">Project Title</h5>
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
  display: flex;
  flex-direction: column;
  padding-left: 0.7em;
  align-items: start;
  border-radius: 0;
  justify-content: start;
}

#tree-container > * {
  width: inherit;
}

</style>
