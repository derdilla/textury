<script setup lang="ts">
import Editor from "@/components/editor/TextEditor.vue";
import EditOutput from "@/components/editor/EditOutput.vue";
import ProjectPanel from "@/components/editor/ProjectPanel.vue";
import { nextTick, computed, onMounted, ref, watch } from "vue";
import Split from 'split-grid';
import { useEditorLayoutStore } from "@/stores/editorLayout";
import { useEditorSession } from "@/composables/editorSession";

const editorLayoutStore = useEditorLayoutStore();
useEditorSession();

const sidebar = ref();
const editor = ref();
const output = ref();
const gutterLeft = ref();
const gutterRight = ref();

const gridColumns = computed(() => {
  if (editorLayoutStore.sidebarHidden.value && editorLayoutStore.outputHidden.value) {
    return "1fr"; // Editor
  } else if (!editorLayoutStore.sidebarHidden.value && !editorLayoutStore.outputHidden.value) {
    return "250px 15px 1fr 15px 1fr"; // Sidebar, Gutter, Editor, Gutter, Output
  } else if (!editorLayoutStore.sidebarHidden.value && editorLayoutStore.outputHidden.value) {
    return "250px 15px 1fr"; // Sidebar, Gutter, Editor
  } else {
    return "1fr 15px 1fr"; // Editor, Gutter, Output
  }
});

let splitInstance: any = null;

const initSplit = () => {
  if (splitInstance) {
    splitInstance.destroy();
  }

  const activeGutters = [];

  if (!editorLayoutStore.sidebarHidden.value && !editorLayoutStore.outputHidden.value) {
    activeGutters.push({
      track: 1,
      element: gutterLeft.value,
    });
    activeGutters.push({
      track: 3,
      element: gutterRight.value,
    });
  } else if ((!editorLayoutStore.sidebarHidden.value && editorLayoutStore.outputHidden.value) || (editorLayoutStore.sidebarHidden.value && !editorLayoutStore.outputHidden.value)) {
    activeGutters.push({
      track: 1,
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
});

watch(editorLayoutStore.sidebarHidden, async () => {
  await nextTick();
  initSplit();
});

watch(editorLayoutStore.outputHidden, async () => {
  await nextTick();
  initSplit();
});
</script>

<template>
  <main class="responsive no-padding no-margin max">
    <div class="draggable-grid no-space" :style="{ gridTemplateColumns: gridColumns }" style="height: 100vh; max-height: 100vh">

      <!--Project Sidebar-->
      <div v-if="!editorLayoutStore.sidebarHidden.value" class="no-padding no-margin grid-item" ref="sidebar">
        <ProjectPanel class="panel active left no-margin no-padding" />
      </div>

       <!--Gutter Left-->
      <div v-if="!editorLayoutStore.sidebarHidden.value && !editorLayoutStore.outputHidden.value" class="gutter-col" :style="{ gridColumn: 2 }" ref="gutterLeft">
        <div class="drag-indicator"></div>
      </div>

      <!--Text Editor-->
      <div class="no-padding no-margin grid-item" ref="editor">
        <Editor />
      </div>

      <!--Gutter Right-->
      <div v-if="!editorLayoutStore.sidebarHidden.value || !editorLayoutStore.outputHidden.value" class="gutter-col" :style="{ gridColumn: (!editorLayoutStore.sidebarHidden.value && !editorLayoutStore.outputHidden.value) ? 4 : 2 }" ref="gutterRight">
        <div class="drag-indicator"></div>
      </div>

      <!--Output Viewer-->
      <div v-if="!editorLayoutStore.outputHidden.value" class="no-padding no-margin grid-item" ref="output">
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