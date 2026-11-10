<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import ConfirmationModal from "@/shared/components/composites/ConfirmationModal.vue";
import ProjectShareModal from "@/features/projects/components/modals/ProjectShareModal.vue";
import ChangeProjectPermissionsModal from "@/features/projects/components/modals/ChangeProjectPermissionModal.vue";
import UserListRow, { type UserBadge, type MenuItem } from "@/features/settings/components/custom/rows/UserListRow.vue";
import { useProjectMembersStore } from "@/features/projects/stores/projectMembersStore";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import { useContactsStore } from "@/shared/stores/contactsStore";
import { useAuthStore } from "@/shared/stores/authStore";
import type { ProjectId, ProjectMemberInfo, ProjectPermissionLevel } from "@/api/types";

const props = defineProps<{
  projectId: ProjectId;
}>();

const membersStore = useProjectMembersStore();
const projectsStore = useProjectsStore();
const contactsStore = useContactsStore();
const authStore = useAuthStore();

const currentProject = computed(() => projectsStore.getProjectById(props.projectId));
const ownerId = computed(() => currentProject.value?.owner);
const currentUserId = computed(() => authStore.state.user?.id);

const isOwner = computed(() => {
  return !!ownerId.value && !!currentUserId.value && ownerId.value === currentUserId.value;
});

const rawMembers = computed<ProjectMemberInfo[]>(() => membersStore.getMembers(props.projectId));

const formattedMembers = computed(() =>
  rawMembers.value.map((member) => ({
    id: member.id,
    name: contactsStore.getUsername(member.id, member.id),
    raw: member,
  }))
);

const selectedMember = ref<ProjectMemberInfo | null>(null);

const isInviteModalOpen = ref(false);
const isPermissionsModalOpen = ref(false);
const isKickModalOpen = ref(false);
const isTransferModalOpen = ref(false);

onMounted(async () => {
  if (!membersStore.isInitialized(props.projectId)) {
    await membersStore.fetchMembers(props.projectId);
  }
});

function handleEditPermissions(member: ProjectMemberInfo) {
  selectedMember.value = member;
  isPermissionsModalOpen.value = true;
}

function handleTransferOwnership(member: ProjectMemberInfo) {
  selectedMember.value = member;
  isTransferModalOpen.value = true;
}

function handleKick(member: ProjectMemberInfo) {
  selectedMember.value = member;
  isKickModalOpen.value = true;
}

function getPermissionLabel(perm: ProjectPermissionLevel): string {
  const permission = String(perm).toLowerCase();
  if (permission === 'view') return 'Viewer';
  if (permission === 'compile') return 'Compile';
  if (permission === 'edit') return 'Editor';
  return String(perm);
}

function getBadges(item: { raw: ProjectMemberInfo }): UserBadge[] {
  // If the user is the owner use the badge 'owner' instead of the permission level.
  if (ownerId.value && item.raw.id === ownerId.value) {
    return [{ label: "Owner" }];
  }
  return [{ label: getPermissionLabel(item.raw.permissions) }];
}

function getMenuItems(item: { raw: ProjectMemberInfo }): MenuItem[] {
  const isTargetOwner = ownerId.value && item.raw.id === ownerId.value;

  const items: MenuItem[] = [
    {
      id: "permissions",
      label: "Change Permissions",
      icon: "admin_panel_settings",
      onClick: () => handleEditPermissions(item.raw),
    },
  ];

  // Transfer Ownership and Kick should only be shown for non-owners.
  if (!isTargetOwner) {
    items.push({
      id: "transfer",
      label: "Transfer Ownership",
      icon: "swap_horiz",
      onClick: () => handleTransferOwnership(item.raw),
    });
    items.push({
      id: "divider",
      label: "",
      isDivider: true,
    });
    items.push({
      id: "kick",
      label: "Kick Member",
      icon: "person_remove",
      isDestructive: true,
      onClick: () => handleKick(item.raw),
    });
  }

  return items;
}

async function handleSavePermissions(permissions: ProjectPermissionLevel) {
  if (!selectedMember.value) return;
  await membersStore.updatePermissionLevel(props.projectId, selectedMember.value.id, permissions);
}

async function confirmKick() {
  if (!selectedMember.value) return;
  await membersStore.kickMember(props.projectId, selectedMember.value.id);
}

async function confirmTransfer() {
  if (!selectedMember.value) return;
  await membersStore.transferOwnership(props.projectId, selectedMember.value.id);
}
</script>

<template>
  <div>
    <UserListRow
      :users="formattedMembers"
      :is-loading="membersStore.state.loadingProjects[props.projectId]"
      :show-header="isOwner"
      title="Project Members"
      description="Manage members, permission levels, and project ownership."
      :get-badges="getBadges"
      :get-menu-items="isOwner ? getMenuItems : undefined"
    >
      <template #header-actions>
        <!-- Invite Member Button -->
        <button v-if="isOwner" class="no-round primary" @click="isInviteModalOpen = true">
          <i>person_add</i>
          <span>Invite Member</span>
        </button>
      </template>
    </UserListRow>

    <!-- Invite Modal -->
    <ProjectShareModal v-model:open="isInviteModalOpen" :project-id="projectId" />

    <!-- Change Permissions Modal -->
    <ChangeProjectPermissionsModal
      v-model:open="isPermissionsModalOpen"
      :member="selectedMember"
      @save="handleSavePermissions"
    />

    <!-- Kick Confirmation Modal -->
    <ConfirmationModal
      v-model:open="isKickModalOpen"
      title="Kick User?"
      :message="selectedMember ? `Are you sure you want to remove ${contactsStore.getUsername(selectedMember.id, selectedMember.id)} from this project?` : ''"
      confirmLabel="Kick User"
      cancelLabel="Cancel"
      variant="error"
      :onConfirm="confirmKick"
    />

    <!-- Transfer Ownership Confirmation Modal -->
    <ConfirmationModal
      v-model:open="isTransferModalOpen"
      title="Transfer Ownership?"
      :message="selectedMember ? `Are you sure you want to transfer project ownership to ${contactsStore.getUsername(selectedMember.id, selectedMember.id)}?` : ''"
      confirmLabel="Transfer"
      cancelLabel="Cancel"
      variant="error"
      :onConfirm="confirmTransfer"
    />
  </div>
</template>
