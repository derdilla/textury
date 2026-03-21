<script setup lang="ts">
//import FileMetadata from '@/services/projectApi';
import { computed, ref } from 'vue';

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

if (props.root != null) console.log(props.root.file_type);
</script>

<template>
  <details v-if="isDirectory" @contextmenu="openContextMenu">
    <summary class="max"><i>folder</i>{{ props.root.name }}</summary>
    <ul v-for="child in children" :key="child.name">
      <li>
        <FileTree :root=child />
      </li>
    </ul>
  </details>
  <div v-else @contextmenu="openContextMenu">
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

details>ul {
  padding-left: 1em;
}

details summary>* {
  display: inline;
}

summary {
  list-style-type: revert;
}
</style>
