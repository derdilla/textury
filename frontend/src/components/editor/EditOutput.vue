<script setup lang="ts">
import { useEditorLayoutStore } from "@/stores/editorLayout";
import { useCompilerOutputStore } from "@/stores/compilerOutput";

const editorLayoutStore = useEditorLayoutStore();
const compilerOutputStore = useCompilerOutputStore();
</script>

<template>
  <div v-if="!editorLayoutStore.showLog.value" class="max" style="height: 100%">
    <embed :src="compilerOutputStore.state.lastCompiledUrl"
           type="application/pdf"
           height="100%"
           width="100%">
  </div>
  <div v-if="editorLayoutStore.showLog.value" class="log-box right-margin">
    <h6 class="bold vertical-margin log-title">Compilation log:</h6>
    <pre class="log bottom-margin square no-border scroll">{{ compilerOutputStore.state.log }}</pre>
  </div>
</template>

<style scoped>
.log-box {
  display: flex;
  flex-flow: column;
  height: 100vh;
}

.log-title {
  flex: 0 1 auto;
  padding-top: 3px;
  height: 2.5rem;
}

.log {
  flex: 1 1 auto;
}
</style>
