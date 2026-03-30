<script setup lang="ts">
import Editor from "@/components/TextEditor.vue";
import EditOutput from "@/components/EditOutput.vue";
import ProjectPanel from "@/components/ProjectPanel.vue";
import { nextTick, computed, onMounted, ref, watch } from "vue";
import Split from 'split-grid';
import { useRoute } from "vue-router";
import { editCtrl } from "@/services/editorController.ts";
import {whoami} from "@/services/managementApi.ts";
import router from "@/router";

const route = useRoute()

const sidebar = ref();
const editor = ref();
const output = ref();
const gutterLeft = ref();
const gutterRight = ref();

const gridColumns = computed(() => {
  return editCtrl.sidebarHidden
    ? "1fr 15px 1fr"             // Editor, Gutter, Output
    : "250px 15px 1fr 15px 1fr"; // Sidebar, Gutter, Editor, Gutter, Output
});

editCtrl.openProject = route.params.id as string;
watch(
  () => route.params.id,
  (newId) => {
    editCtrl.openProject = newId as string;
  },
);

let splitInstance: any = null;

const initSplit = () => {
  if (splitInstance) {
    splitInstance.destroy();
  }

  const activeGutters = [];

  if (editCtrl.sidebarHidden) {
    activeGutters.push({
      track: 1,
      element: gutterRight.value,
    });
  } else {
    activeGutters.push({
      track: 1,
      element: gutterLeft.value,
    });
    activeGutters.push({
      track: 3,
      element: gutterRight.value,
    });
  }

  splitInstance = Split({
    columnGutters: activeGutters,
    minSize: 100
  });
};

onMounted(() => {
  initSplit();
})

watch(() => editCtrl.sidebarHidden, async () => {
  await nextTick();
  initSplit();
});

whoami().then(async (user) => {
  if (user === null) await router.push('/login')
});
</script>

<template>
  <main class="responsive no-padding no-margin max">
    <div class="draggable-grid no-space" :style="{ gridTemplateColumns: gridColumns }" style="height: 100vh; max-height: 100vh">

      <!--Project Sidebar-->
      <div v-if="!editCtrl.sidebarHidden" class="no-padding no-margin grid-item" ref="sidebar">
        <ProjectPanel class="panel active left no-margin no-padding" />
      </div>

       <!--Gutter Left-->
      <div v-if="!editCtrl.sidebarHidden" class="gutter-col" :style="{ gridColumn: 2 }" ref="gutterLeft">
        <div class="drag-indicator"></div>
      </div>

      <!--Text Editor-->
      <div class="no-padding no-margin grid-item" ref="editor">
        <Editor />
      </div>

      <!--Gutter Right-->
      <div class="gutter-col" :style="{ gridColumn: editCtrl.sidebarHidden ? 2 : 4 }" ref="gutterRight">
        <div class="drag-indicator"></div>
      </div>

      <!--Output Viewer-->
      <div class="no-padding no-margin grid-item" ref="output">
        <EditOutput />
      </div>

    </div>
  </main>
</template>

<style scoped>
.draggable-grid {
  display: grid;
  height: 100vh;
}

.grid-item {
  min-width: 0;
  overflow: hidden;
}

.gutter-col {
  position: relative;
  grid-row: 1/-1;
  cursor: col-resize;
  background-color: var(--surface);
}

.drag-indicator {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 5px;
  height: 80px;
  border-radius: 5px;
  background-color: var(--surface-container-highest);
}

.hidden {
  display: none;
}
</style>