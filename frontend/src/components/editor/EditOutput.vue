<script setup lang="ts">
import {editCtrl} from "@/services/editorController.ts";
</script>

<template>
  <div v-if="!editCtrl.showLog" class="max" style="height: 100%">
    <embed :src="editCtrl.lastCompiled"
           type="application/pdf"
           height="100%"
           width="100%">
  </div>
  <div v-if="editCtrl.showLog" class="log-box right-margin">
    <h6 class="bold vertical-margin log-title">Compilation log:</h6>
    <pre class="log bottom-margin square no-border scroll">{{ editCtrl.log }}</pre>
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
