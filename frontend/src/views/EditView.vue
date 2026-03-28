<script setup lang="ts">

import Editor from "@/components/TextEditor.vue";
import EditOutput from "@/components/EditOutput.vue";
import ProjectPanel from "@/components/ProjectPanel.vue";
import { computed, onMounted, provide, reactive, ref, watch } from "vue";
import Split from 'split-grid';
import { useRoute } from "vue-router";
import { ProjectApi } from "@/services/projectApi";
import { editCtrl } from "@/services/editorController.ts";

const route = useRoute()

const editor = ref();
const output = ref();
const gutter = ref();

editCtrl.openProject = route.params.id as string;
watch(
  () => route.params.id,
  (newId) => {
    editCtrl.openProject = newId as string;
  },
);

onMounted(() => {
  Split({
    columnGutters: [{
      track: 1,
      element: gutter.value,
    }],
  });

})
</script>

<template>
  <ProjectPanel v-if="!editCtrl.sidebarHidden" class="panel active left no-margin no-padding" />
  <main class="responsive no-padding no-margin max">
    <div class="draggable-grid no-space" style="height: 100vh; max-height: 100vh">
      <div class="no-padding no-margin" ref="editor">
        <Editor />
      </div>
      <div class="gutter-col gutter-col-1" ref="gutter"></div>
      <div class="no-padding no-margin" ref="output">
        <EditOutput />
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
