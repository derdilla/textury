<script setup lang="ts">
import { ProjectApi, type FileMetadata } from '@/services/projectApi';
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

async function openContextMenu(e: Event) {
  // FIXME: throws error for bottom entries
  e.preventDefault();
  menu.value.classList.toggle("active");
}

function setAsOpen() {
  console.log(`Setting to: ${props.root.whole_path}`);
  editCtrl.openFile= props.root.whole_path;
}

async function dropHandler(e: DragEvent) {
  console.log(e);
  e.preventDefault();
  if (!e.dataTransfer) return;
  const fileItems = [...e.dataTransfer.items].map(
      (item: DataTransferItem) => item.getAsFile()
    ).filter(
      (item) => item != null,
    );
    if (fileItems.length > 0 && editCtrl.openProject != null) {
      const api = new ProjectApi(editCtrl.openProject);
      for (const file of fileItems) {
        const path = props.root.whole_path + file.name;
        const res = await api.createOrUpdateFile(path, file.stream());
        if (!res.success) {
          console.error(`Error uploading '${file.name}': ${res.message}`);
        }
      }
      const index = await api.index();
      editCtrl.dirIndex = index;
    }
}

</script>

<template>
  <details v-if="isDirectory" @drop="dropHandler" @contextmenu="openContextMenu">
    <summary><i>folder</i>{{ props.root.name }}</summary>
    <ul v-for="child in children" :key="child.name" @drop="dropHandler">
      <input type="file" id="file-input" multiple accept="*" style="display: none" />
      <li>
        <FileTree :root=child />
      </li>
    </ul>
  </details>
  <div v-else @click="setAsOpen" @contextmenu="openContextMenu" class="file left"
    :class="{ secondary: editCtrl?.openFile === props.root.whole_path }">
    <i>article</i>
    <span>{{ props.root.name }}</span>
    <!-- <button class="transparent small right">
      <i>more_vert</i>
      <span class="tooltip left">Options</span>
      </button> -->
    <menu ref="menu" class="left">
      <li class="min"><i>delete</i> Delete</li>
    </menu>
  </div>
</template>

<style scoped>

details {
  display: flex;
  align-items: start;
  flex-flow: column;
  width: inherit;
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
  display: flex;
  justify-content: start
}

.drop-zone {
  cursor: pointer;
}
</style>
