<script setup lang="ts">
import { ref, computed, watch } from "vue";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";
import { useAuthStore } from "@/shared/stores/authStore";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import { useProjectMembersStore } from "@/features/projects/stores/projectMembersStore";
import { useContactsStore } from "@/shared/stores/contactsStore";
import type { ProjectId, UserId } from "@/api/types";

const open = defineModel<boolean>("open", { default: false });
const props = defineProps<{
  projectId: ProjectId;
}>();

const authStore = useAuthStore();
const projectsStore = useProjectsStore();
const contactsStore = useContactsStore();
const membersStore = useProjectMembersStore();

const project = computed(() => projectsStore.getProjectById(props.projectId));

const isOwner = computed(() => {
  const ownerId = project.value?.owner;
  const userId = authStore.state.user?.id;
  return ownerId && userId && ownerId == userId;
});

const otherMembers = computed<UserId[]>(() => {
  const allMembers = project.value?.members || [];
  return allMembers.filter((id) => id != authStore.state.user?.id);
});

const step = ref<"select-owner" | "confirm" | "error">("confirm");
const selectedNewOwnerId = ref<UserId | null>(null);
const isLoading = ref(false);

watch(open, async (isOpen) => {
  if (isOpen) {
    selectedNewOwnerId.value = null;
    if (isOwner.value) {
      if (otherMembers.value.length === 0) {
        step.value = "error";
      } else {
        step.value = "select-owner";
        selectedNewOwnerId.value = otherMembers.value[0] ?? null;
      }
    } else {
      step.value = "confirm";
    }
  }
});

function handleContinue() {
  step.value = "confirm";
}

async function handleLeave() {
  isLoading.value = true;
  try {
    if (selectedNewOwnerId.value) {
      const res = await membersStore.transferOwnership(props.projectId, selectedNewOwnerId.value);
      if (!res.success) {
        isLoading.value = false;
        return;
      }
    }

    await projectsStore.leaveProject(props.projectId);
    open.value = false;
  } catch (err) {
    console.error("Error leaving project:", err);
  } finally {
    isLoading.value = false;
  }
}
</script>

<template>
  <ModalPopover v-model:open="open" max-width="500px">
    <!-- Header -->
    <template #header>
      <h5 class="bold dialog-title no-margin">
        {{ step === 'select-owner' ? 'Transfer Ownership' : step === 'error' ? 'Cannot Leave Project' : 'Leave Project?' }}
      </h5>
    </template>

    <!-- Content -->
    <div class="modal-body">
      <template v-if="step === 'error'">
        <p class="no-margin error-text">
          As the owner and only member of this project, you can only delete it.
        </p>
      </template>

      <template v-else-if="step === 'select-owner'">
        <p class="no-margin">
          As the owner, you must transfer ownership to another member before you can leave.
        </p>

        <div class="field label border suffix small">
          <select v-model="selectedNewOwnerId" class="no-round active">
            <option v-for="userId in otherMembers" :key="userId" :value="userId">
              {{ contactsStore.getUsername(userId, userId) }}
            </option>
          </select>
          <label class="active">New Owner</label>
          <i class="select-icon">keyboard_arrow_down</i>
        </div>
      </template>

      <template v-else>
        <p class="no-margin">
          {{ isOwner && selectedNewOwnerId
            ? `Ownership will be transferred and you will permanently lose access to this project.`
            : 'You will permanently lose access to this project.' }}
        </p>
      </template>
    </div>

    <!-- Actions -->
    <template #actions>
      <div class="row right-align max">
        <button
          v-if="step === 'confirm' && isOwner"
          class="border no-round back-button"
          @click="step = 'select-owner'"
        >
          Back
        </button>

        <div class="row gap">
          <button class="border no-round" @click="open = false">
            {{ step === 'error' ? 'Close' : 'Cancel' }}
          </button>

          <button
            v-if="step === 'select-owner'"
            class="primary no-round"
            :disabled="!selectedNewOwnerId"
            @click="handleContinue"
          >
            Continue
          </button>

          <button
            v-if="step === 'confirm'"
            class="error no-round"
            :disabled="isLoading"
            @click="handleLeave"
          >
            {{ isLoading ? 'Leaving...' : 'Leave Project' }}
          </button>
        </div>
      </div>
    </template>
  </ModalPopover>
</template>

<style scoped>
.modal-body {
  display: flex;
  flex-direction: column;
  width: 100%;
  gap: 1rem;
  margin-top: 0.5rem;
  margin-bottom: 1rem;
}

.back-button {
  margin-right: auto;
}

.select-icon {
  transform: translate(5px, 0px);
}
</style>
