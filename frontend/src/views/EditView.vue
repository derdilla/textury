<script setup lang="ts">

import Editor from "@/components/Editor.vue";
import EditOutput from "@/components/EditOutput.vue";
import ProjectPanel from "@/components/ProjectPanel.vue";
import {onMounted, ref} from "vue";
import Split from 'split-grid';

const editor = ref();
const output = ref();
const gutter = ref();
const hideSidebar = ref(true);

function toggleSidebar() {
  hideSidebar.value = !hideSidebar.value;
}

onMounted(() => {
  Split({
    columnGutters: [{
      track: 1,
      element: gutter.value,
    }],
  })
})
</script>

<template>
  <ProjectPanel class="panel active left no-margin no-padding" :class="{ hidden: hideSidebar }"
                :toggleMenu="toggleSidebar"/>
  <main class="responsive no-padding no-margin max">
    <div class="draggable-grid no-space">
      <div class="no-padding no-margin" style="max-height: 100vh" ref="editor">
        <Editor :showMenuOpener="hideSidebar" :toggleMenu="toggleSidebar"/>
      </div>
      <div class="gutter-col gutter-col-1" ref="gutter"></div>
      <div class="no-padding no-margin" ref="output">
        <EditOutput/>
      </div>
    </div>
  </main>
</template>

<style scoped>
.draggable-grid {
  display: grid;
  grid-template-columns: 1fr 3px 1fr;
}

.gutter-col {
  grid-row: 1/-1;
  cursor: col-resize;
}

.gutter-col-1 {
  grid-column: 2;
}

.hidden {
  display: none;
}
</style>