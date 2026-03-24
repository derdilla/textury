<script setup lang="ts">
import type { FileMetadata } from '@/services/projectApi';
import { computed, ref } from 'vue';
import {editCtrl} from "@/services/editorController.ts";

const props = defineProps<{ root: FileMetadata }>();
const isDirectory = computed(() => {
  return props.root.file_type.Directory != null;
});
const children = computed(() => {
  if (props.root.file_type.Directory != null
    && props.root.file_type.Directory.children != null) {
    return props.root.file_type.Directory.children
  }
  return [];
});

const menu = ref();

async function openContextMenu(e) {
  e.preventDefault();
  menu.value.classList.toggle("active");
  // FIXME: menu doesn't actually open
}

function setAsOpen() {
  console.log(`Setting to: ${props.root.whole_path}`);
  editCtrl.openFile= props.root.whole_path;
}
</script>

<template>
  <details v-if="isDirectory" @contextmenu="openContextMenu">
    <summary><i>folder</i>{{ props.root.name }}</summary>
    <ul v-for="child in children" :key="child.name">
      <li>
        <FileTree :root=child />
      </li>
    </ul>
  </details>
  <div v-else @click="setAsOpen" @contextmenu="openContextMenu" class="file"
    :class="{ secondary: editCtrl?.openFile === props.root.whole_path }">
    <i>article</i>
    <span>{{ props.root.name }}</span>
    <!-- <button class="transparent small right">
      <i>more_vert</i>
      <span class="tooltip left">Options</span>
      </button> -->
  </div>
  <menu ref="menu" id="menu">
    <li class="min"><i>delete</i> Delete</li>
  </menu>
</template>

<style scoped>
details {
  display: flex;
  align-items: start;
  flex-flow: column;
}

details ul {
  padding-left: 1em;
  margin-top: 0 !important;
  margin-bottom: 0;
}

details summary>* {
  display: inline;
}

.file {
  cursor: pointer;
  padding-right: 4px;
}
</style>
