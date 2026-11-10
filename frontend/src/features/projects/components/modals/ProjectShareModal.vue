<script setup lang="ts">
import { ref, computed, watch } from "vue";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";
import CodeOutputField from "@/shared/components/composites/CodeOutputField.vue";
import PermissionSelector from "@/shared/components/composites/PermissionSelector.vue";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import { formatExpiryDuration } from "@/shared/utils/formatters";
import type { ProjectId, ProjectPermissionLevel } from "@/api/types";

const open = defineModel<boolean>("open", { default: false });
const props = defineProps<{
  projectId: ProjectId;
}>();

const projectsStore = useProjectsStore();

const step = ref<"select-role" | "show-link">("select-role");
const selectedPermission = ref<ProjectPermissionLevel>("edit");

const shareUrl = ref("");
const expiresMinutes = ref<number | null>(null);
const isLoading = ref(false);
const errorMessage = ref("");

const permissionTitles: Record<ProjectPermissionLevel, string> = {
  edit: "Editor",
  compile: "Compile",
  view: "Viewer",
};

watch(open, (isOpen) => {
  if (isOpen) {
    step.value = "select-role";
    selectedPermission.value = "edit";
    shareUrl.value = "";
    expiresMinutes.value = null;
    errorMessage.value = "";
  }
});

async function handleContinue() {
  step.value = "show-link";
  isLoading.value = true;
  errorMessage.value = "";
  try {
    const res = await projectsStore.createShareToken(props.projectId, selectedPermission.value);
    if (res.success && res.data) {
      shareUrl.value = `${window.location.origin}/acceptInvite/${res.data.token}`;
      expiresMinutes.value = res.data.expires;
    } else {
      errorMessage.value = res.error || "Failed to generate share link.";
    }
  } catch {
    errorMessage.value = "Failed to generate share link.";
  } finally {
    isLoading.value = false;
  }
}

const formattedExpiry = computed(() => formatExpiryDuration(expiresMinutes.value));
</script>

<template>
  <ModalPopover v-model:open="open" max-width="500px">
    <template #header>
      <div class="max dialog-header-content">
        <h5 class="bold dialog-title no-margin">
          {{ step === "select-role" ? "Permission Level" : "Invite Member" }}
        </h5>
      </div>
    </template>

    <!-- Content -->
    <div class="modal-body-content">
      <div v-if="errorMessage" class="chip max no-round error-text grid bottom-margin error-box">
        <i class="s1">error</i>
        <span class="s11 wrap text-wrap">{{ errorMessage }}</span>
      </div>

      <template v-if="step === 'select-role'">
        <p class="on-surface no-margin left-align">
          Choose the permission level for the generated invite link:
        </p>

        <PermissionSelector v-model="selectedPermission" />
      </template>

      <template v-else>
        <div v-if="isLoading" class="center-align padding">
          <progress class="circle"></progress>
        </div>

        <template v-else-if="shareUrl">
          <p class="on-surface no-margin left-align">
            Share this invitation link to join the project:
          </p>

          <CodeOutputField
            :value="shareUrl"
            label="URL"
            share-title="Join Project"
            share-text="Here is the invitation link to join the project:"
          />

          <div class="row no-margin middle-align expiry-row" style="gap: 1.25rem;">
            <div class="row no-margin middle-align" style="gap: 0.35rem;">
              <i class="small text-secondary">admin_panel_settings</i>
              <span class="small-text text-secondary">{{ permissionTitles[selectedPermission] }}</span>
            </div>
            <div v-if="expiresMinutes" class="row no-margin middle-align" style="gap: 0.35rem;">
              <i class="small text-secondary">schedule</i>
              <span class="small-text text-secondary">{{ formattedExpiry }}</span>
            </div>
          </div>
        </template>
      </template>
    </div>

    <!-- Actions -->
    <template #actions v-if="step === 'select-role'">
      <div class="row right-align">
        <div class="row gap">
          <button class="border no-round" @click="open = false">Cancel</button>
          <button class="primary no-round" @click="handleContinue">Continue</button>
        </div>
      </div>
    </template>
  </ModalPopover>
</template>

<style scoped>
.dialog-header-content {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.15rem;
  width: 100%;
  min-width: 0;
}

.modal-body-content {
  display: flex;
  flex-direction: column;
  width: 100%;
  margin-top: 0.5rem;
  margin-bottom: 0.5rem;
  gap: 1rem;
}

.expiry-row {
  margin-top: -0.25rem;
}

.error-box {
  overflow: hidden;
  padding-top: 0.6rem;
  padding-bottom: 0.6rem;
}

.text-secondary {
  opacity: 0.7;
}
</style>
