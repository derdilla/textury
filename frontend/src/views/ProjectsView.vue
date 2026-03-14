<script setup lang="ts">
import {ref} from "vue";
import CreateProjectForm from "@/components/CreateProjectForm.vue";
import {listProjects} from "@/services/managementApi.ts";

const projects = ref([]);
const addDialog = ref<HTMLDialogElement>();
const addForm = ref<CreateProjectForm>();

async function updateProjectList() {
  const res = await listProjects();
  projects.value = [];
  for (let e of res.projects) {
    projects.value[projects.value.length] = {
      title: e.id,
      id: e.id,
      author: 'unknown'
    };
  }
}

function showDialog() {
  addDialog.value?.show();
}

async function submitDialog() {
  const success = await addForm.value.submit();
  if (success) {
    addDialog.value?.close();
    await updateProjectList();
  }
}

function cancelDialog() {
  addForm.value?.clear();
  addDialog.value?.close();
}

updateProjectList();

</script>

<template>
  <header class="fixed">
    <nav>
      <h6 class="max">Projects</h6>

      <button @click="showDialog" class="outline"><i>add</i> <span>Create</span></button>
    </nav>
  </header>
  <main>

    <main>
      <dialog class="responsive" ref="addDialog">
        <nav>
          <button @click="cancelDialog" class="circle transparent"><i>close</i></button>
          <h6 class="max">Add Project</h6>

          <button @click="submitDialog" class="outline"><i>done</i> <span>Save</span></button>
        </nav>
        <CreateProjectForm ref="addForm"/>
      </dialog>


      <ul class="list border" v-for="proj in projects">
        <li>
          <RouterLink :to="`/projects/${proj.id}`" class="max left-align">
            <div>
              <h6 class="small">{{ proj.title }}</h6>
              <sub class="small italic">By {{ proj.author }}</sub>
            </div>
          </RouterLink>

          <button class="circle transparent">
            <i>more_vert</i>
            <span class="tooltip left">Options</span>
          </button>
          <menu class="no-wrap bottom left">
            <li class="min"><i>delete</i> Delete</li>
            <li><i>person_add</i> Invite user</li>
            <li><i>manage_accounts</i> View and remove users</li>
          </menu>
        </li>
      </ul>
    </main>
  </main>
</template>
