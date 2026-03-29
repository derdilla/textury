<script setup lang="ts">
import { ref } from "vue";
import CreateProjectForm from "@/components/CreateProjectForm.vue";
import UserInviter from "@/components/UserInviter.vue";
import { deleteProject, listProjects, share } from "@/services/managementApi.ts";

const projects = ref<ProjectSummary[]>([]);
const addDialog = ref<HTMLDialogElement>();
const addForm = ref<CreateProjectForm>();

async function updateProjectList() {
  const res = await listProjects();
  if (res == null) return;
  projects.value = res.projects;
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

const shareUrl = ref("");
const shareDialog = ref<HTMLDialogElement>();
async function shareProject(projectId: string) {
  const shareToken = await share(projectId);
  if (!shareToken) return;
  shareUrl.value = `${location.protocol}//${location.hostname}:${location.port}/acceptInvite/${shareToken}`
  shareDialog.value?.showPopover();
}

updateProjectList();

</script>

<template>
  <header class="fixed">
    <nav>
      <h6 class="max">Projects</h6>

      <UserInviter />
      <button @click="showDialog"><i>add</i> <span>Create</span></button>
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
        <CreateProjectForm ref="addForm" />
      </dialog>


      <ul class="list border" v-for="proj in projects" :key="proj.id">
        <li>
          <RouterLink :to="`/projects/${proj.id}`" class="max left-align">
            <div>
              <h6 class="small">{{ proj.title }}</h6>
              <sub class="small italic">By User {{ proj.owner }}</sub>
            </div>
          </RouterLink>

          <button class="circle transparent">
            <i>more_vert</i>
            <span class="tooltip left">Options</span>
          </button>
          <menu class="no-wrap bottom left">
            <li @click="deleteProject(proj.id)" class="min"><i>delete</i> Delete</li>
            <li @click="shareProject(proj.id)"><i>person_add</i> Invite user</li>
            <li><i>manage_accounts</i> View and remove users</li>
          </menu>
        </li>
      </ul>
      <dialog popover ref="shareDialog">
        <p>Tell the invited person to go to this url:</p>
        <pre>{{ shareUrl }}</pre>
      </dialog>
    </main>
  </main>
</template>
