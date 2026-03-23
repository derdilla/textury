<script setup lang="ts">

import Editor from "@/components/TextEditor.vue";
import EditOutput from "@/components/EditOutput.vue";
import ProjectPanel from "@/components/ProjectPanel.vue";
import { computed, onMounted, provide, ref, watch } from "vue";
import Split from 'split-grid';
import type { EditorController } from "@/services/editorControllerType";
import { useRoute } from "vue-router";
import { ProjectApi } from "@/services/projectApi";

const route = useRoute()

const editor = ref();
const output = ref();
const gutter = ref();

const openProject = ref(route.params.id as string);
const editCtrl: EditorController = {
  sidebarHidden: ref(false),
  toggleSidebar() {
    this.sidebarHidden.value = !this.sidebarHidden.value;
  },
  openProject: openProject,
  api: computed(() => new ProjectApi(openProject.value)),
  openFile: ref(null),
};
provide('editorController', editCtrl);

watch(
  () => route.params.id,
  (newId, _oldId) => {
    editCtrl.openProject.value = newId as string;
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
  <ProjectPanel v-if="!editCtrl.sidebarHidden.value" class="panel active left no-margin no-padding" />
  <main class="responsive no-padding no-margin max">
    <div class="draggable-grid no-space">
      <div class="no-padding no-margin" style="max-height: 100vh" ref="editor">
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
