<script setup lang="ts">
import { ProjectApi, FileMetadata } from '@/services/projectApi';
import { ref } from 'vue';
import FileTree from './FileTree.vue';

defineProps(['toggleMenu'])

const addDialog = ref()
const fileName = defineModel('fileName')

const dirIndex = ref<FileMetadata | null>(null);
const api = new ProjectApi('project1');
api.index().then((index) => {
  if (index) { dirIndex.value = index }
});


function openDialog() {
  addDialog.value?.show();
  fileName.value = '';
}

function closeDialog() {
  addDialog.value?.close();
}

async function addFile() {
  await api.createOrUpdateFile(fileName.value);
  closeDialog();
  dirIndex.value = await api.index();
}
</script>

<template>
  <nav class="left panel round width-limited">
    <button @click="toggleMenu" class="transparent circle extra small-margin">
      <i>menu</i>
    </button>
    <h5 class="small no-padding no-margin">Project Title</h5>
    <div id="tree-container">
      <FileTree v-if="dirIndex !== null" :root="dirIndex" />
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
  padding-left: 0.7em;
}
</style>
