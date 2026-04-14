<script setup lang="ts">
import { ref } from "vue";
import CreateProjectForm from "@/components/CreateProjectForm.vue";
import UserInviter from "@/components/UserInviter.vue";
import { deleteProject, listProjects, share, whoami, API_ROOT, type ProjectSummary } from "@/services/managementApi.ts";
import { editCtrl } from "@/services/editorController";
import router from "@/router";
import FloatingElement from "@/components/FloatingElement.vue";

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

async function downloadProject(projectId: string) {
  window.open(`${API_ROOT}/project/${projectId}/zip`, '_self')
}

async function deleteProjectAndUpdate(projectId: string) {
  await deleteProject(projectId);
  updateProjectList();
}

updateProjectList();

if (!editCtrl.user) {
  whoami().then(async (user) => {
    editCtrl.user = user;
    if (user == null) await router.push('/login');
  });
}

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
              <sub class="small italic">By {{ proj.owner.name }}</sub>
            </div>
          </RouterLink>

          <FloatingElement align="left">
            <template #button>
              <button tabindex="0" class="square transparent">
                <i>more_vert</i>
              </button>
            </template>
            <template #menu>
              <menu id="dropdown" class="no-wrap no-round active surface-container shadow">
                <li @click="downloadProject(proj.id)" class="no-round small left-align"><i>download</i> Download ZIP
                </li>
                <li v-if="proj.owner.id === editCtrl.user?.id" @click="deleteProjectAndUpdate(proj.id)"
                  class="no-round small left-align error-text"><i>delete</i> Delete</li>
                <li v-if="proj.owner.id === editCtrl.user?.id" @click="shareProject(proj.id)"
                  class="no-round small left-align"><i>person_add</i> Invite user</li>
                <li v-if="proj.owner.id === editCtrl.user?.id" class="no-round small left-align"><i>manage_accounts</i>
                  View and remove users</li>
              </menu>
            </template>
          </FloatingElement>
        </li>
      </ul>
      <dialog popover ref="shareDialog">
        <p>Tell the invited person to go to this url:</p>
        <pre>{{ shareUrl }}</pre>
      </dialog>
    </main>
  </main>
</template>
