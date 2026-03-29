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
  <div v-if="editCtrl.showLog" class="small-padding log-box">
    <h6>Compilation log:</h6>
    <pre class="log fill left-round medium-padding no-border scroll">{{ editCtrl.log }}</pre>
  </div>

</template>

<style scoped>
.log-box {
  display: flex;
  flex-flow: column;
  height: 100vh;
}

.log-box > h6 {
  flex: 0 1 auto;
}

.log {
  margin: .5em;
  margin-right: 0;
  flex: 1 1 auto;
}
</style>
