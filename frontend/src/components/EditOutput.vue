<script setup lang="ts">
import {editCtrl} from "@/services/editorController.ts";
</script>

<template>
  <div style="height: 100vh;">
    <embed :src="editCtrl.lastCompiled"
           type="application/pdf"
           height="100%"
           width="100%">
  </div>

</template>

<style scoped>

</style>
