<script setup lang="ts">
import { computed } from "vue";
import { useEditorLayoutStore } from "@/features/editor/stores/editorLayoutStore";
import { useCompilerOutputStore } from "@/features/editor/stores/compilerOutputStore";

const editorLayoutStore = useEditorLayoutStore();
const compilerOutputStore = useCompilerOutputStore();

const isOverlay = computed(() => editorLayoutStore.isRightOverlay.value);
const showLog = computed(() => editorLayoutStore.showLog.value);
const isHeaderHidden = computed(() => !showLog.value && !isOverlay.value);
</script>

<template>
  <div class="output-panel surface-container">
    <nav
      class="header-container margin row"
      :class="{ 'pdf-header-hidden': isHeaderHidden }"
    >
      <button v-if="isOverlay" @click="editorLayoutStore.closeOutput" class="square transparent">
        <i>close</i>
      </button>
      <h6 v-if="showLog" class="bold no-margin max">Compilation log</h6>
      <div v-else class="max"></div>
    </nav>

    <div v-if="!showLog" class="embed-wrapper max">
      <embed :src="compilerOutputStore.state.lastCompiledUrl"
             type="application/pdf"
             height="100%"
             width="100%">
    </div>

    <div v-if="showLog" class="log-box right-margin">
      <pre class="log bottom-margin square no-border scroll">{{ compilerOutputStore.state.log }}</pre>
    </div>
  </div>
</template>

<style scoped>
.pdf-header-hidden {
  display: none !important;
}

.output-panel {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.header-container {
  height: 2.5rem;
  flex-shrink: 0;
}

.embed-wrapper {
  flex: 1;
  min-height: 0;
  height: 100%;
}

.log-box {
  display: flex;
  flex-flow: column;
  flex: 1;
  min-height: 0;
  height: calc(100vh - 2.5rem);
}

.log {
  flex: 1 1 auto;
}
</style>
