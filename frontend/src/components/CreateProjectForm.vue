<script setup>
import {ref} from "vue";
import {createProject} from "@/services/managementApi.ts";

defineExpose({
  clear,
  submit
})


let idModified = false;
const projectId = defineModel('projectId')
const title = defineModel('title', {
  set(value) {
    if (!idModified) {
      projectId.value = value.replaceAll(' ', '_')
          .replaceAll(/([^a-zA-Z0-9.-_~!$&'()*+,;=:@])/g, '');
    }

    return value;
  }
})

// There is already a project with this id
// Must match <code>[a-zA-Z0-9.-_~!$&'()*+,;=:@]*</code>
const projectIdErr = ref('')

function clear() {
  idModified = false;
  projectIdErr.value = '';
  title.value = '';
  projectId.value = '';
}

async function submit() {
  projectIdErr.value = '';
  const result = await createProject(projectId.value);
  if (result.success) {
    clear();
  } else {
    projectIdErr.value = result.message;
  }
  return result.success;
}
</script>

<template>
  <form class="large-padding" @submit.prevent>
    <div class="field border label">
      <input v-model="title" type="text">
      <label>Title</label>
    </div>
    <div class="field border label">
      <input v-model="projectId" @click="idModified = true" type="text">
      <label>Project id</label>
      <output class="invalid">{{ projectIdErr }}</output>
    </div>
  </form>
</template>