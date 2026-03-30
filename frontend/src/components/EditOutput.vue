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
  <div v-if="editCtrl.showLog" class="log-box">
    <h6>Compilation log:</h6>
    <pre class="log fill square no-border scroll">{{ editCtrl.log }}</pre>
  </div>

</template>

<style scoped>
.log-box {
  display: flex;
  flex-flow: column;
  height: 100vh;
}

.log-box > h6 {
  margin-top: 15px;
  flex: 0 1 auto;
  font-family: Avenir, Montserrat, Corbel, 'URW Gothic', source-sans-pro, sans-serif;
  font-weight: bold;
}

.log {
  margin: 15px;
  margin-left: 0;
  flex: 1 1 auto;
}
</style>
