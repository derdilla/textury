<script setup lang="ts">
import { ref, watch } from "vue";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";
import PermissionSelector from "@/shared/components/composites/PermissionSelector.vue";
import type { ProjectMemberInfo, ProjectPermissionLevel } from "@/api/types";

const open = defineModel<boolean>("open", { default: false });
const props = defineProps<{
  member: ProjectMemberInfo | null;
}>();

const emit = defineEmits<{
  (e: "save", permissions: ProjectPermissionLevel): void;
}>();

const permissions = ref<ProjectPermissionLevel>("view");

watch(open, (isOpen) => {
  if (isOpen && props.member) {
    permissions.value = props.member.permissions;
  }
}, { immediate: true });

function submit() {
  emit("save", permissions.value);
  open.value = false;
}
</script>

<template>
  <ModalPopover v-model:open="open" max-width="500px">
    <template #header>
      <div class="max dialog-header-content">
        <h5 class="bold dialog-title no-margin">Change Permissions</h5>
      </div>
    </template>

    <div class="modal-body-content">
      <p class="on-surface no-margin left-align">
        Select the new permission level for this member:
      </p>

      <PermissionSelector v-model="permissions" />
    </div>

    <template #actions>
      <div class="row right-align">
        <div class="row gap">
          <button class="border no-round" @click="open = false">Cancel</button>
          <button class="primary no-round" @click="submit">
            <i>check</i> Save
          </button>
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
  margin-top: 0.25rem;
  margin-bottom: 0.5rem;
  gap: 0.75rem;
}
</style>
