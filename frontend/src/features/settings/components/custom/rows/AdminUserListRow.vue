<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import ConfirmationModal from "@/shared/components/composites/ConfirmationModal.vue";
import InviteUserModal from "../modals/InviteUserModal.vue";
import EditUserModal from "../modals/EditUserModal.vue";
import ChangeRoleModal from "../modals/ChangeRoleModal.vue";
import UserListRow, { type UserBadge, type MenuItem } from "@/features/settings/components/custom/rows/UserListRow.vue";
import { useUserManagementStore } from "@/features/settings/stores/userManagementStore";
import type { PrivateAccountInfo } from "@/api/types";

const usersManagementStore = useUserManagementStore();

const rawUsers = computed<PrivateAccountInfo[]>(() => usersManagementStore.users.value);

const formattedUsers = computed(() =>
  rawUsers.value.map((user) => ({
    id: user.id,
    name: user.username,
    subtitle: user.email,
    raw: user,
  }))
);

const selectedUser = ref<PrivateAccountInfo | null>(null);

const isInviteModalOpen = ref(false);
const isEditModalOpen = ref(false);
const isRoleModalOpen = ref(false);
const isDeleteModalOpen = ref(false);

onMounted(async () => {
  if (!usersManagementStore.state.isInitialized) {
    await usersManagementStore.fetchUsers();
  }
});

function handleEdit(user: PrivateAccountInfo) {
  selectedUser.value = user;
  isEditModalOpen.value = true;
}

function handleChangeRole(user: PrivateAccountInfo) {
  selectedUser.value = user;
  isRoleModalOpen.value = true;
}

function handleDelete(user: PrivateAccountInfo) {
  selectedUser.value = user;
  isDeleteModalOpen.value = true;
}

function getBadges(item: { raw: PrivateAccountInfo }): UserBadge[] {
  if (item.raw.isAdmin) {
    return [{ label: "Admin" }];
  }
  return [];
}

function getMenuItems(item: { raw: PrivateAccountInfo }): MenuItem[] {
  return [
    {
      id: "edit",
      label: "Edit User",
      icon: "edit",
      onClick: () => handleEdit(item.raw),
    },
    {
      id: "change-role",
      label: "Change Role",
      icon: "admin_panel_settings",
      onClick: () => handleChangeRole(item.raw),
    },
    {
      id: "divider",
      label: "",
      isDivider: true,
    },
    {
      id: "delete",
      label: "Delete",
      icon: "delete",
      isDestructive: true,
      onClick: () => handleDelete(item.raw),
    },
  ];
}

async function handleSaveUser(data: { username: string; email: string }) {
  if (!selectedUser.value) return;
  const userId = selectedUser.value.id;
  if (data.username !== selectedUser.value.username) {
    await usersManagementStore.updateUsername(userId, data.username);
  }
  if (data.email !== selectedUser.value.email) {
    await usersManagementStore.updateEmail(userId, data.email);
  }
}

async function handleSaveRole(isAdmin: boolean) {
  if (!selectedUser.value) return;
  if (isAdmin !== selectedUser.value.isAdmin) {
    await usersManagementStore.updateIsAdmin(selectedUser.value.id, isAdmin);
  }
}

async function confirmDelete() {
  if (!selectedUser.value) return;
  await usersManagementStore.deleteUser(selectedUser.value.id);
}
</script>

<template>
  <div>
    <UserListRow
      :users="formattedUsers"
      :is-loading="usersManagementStore.state.isLoading"
      title="Users"
      description="Manage accounts, roles, and user projects."
      :get-badges="getBadges"
      :get-menu-items="getMenuItems"
    >
      <template #header-actions>
        <button class="no-round primary" @click="isInviteModalOpen = true">
          <i>person_add</i>
          <span>Invite User</span>
        </button>
      </template>
    </UserListRow>

    <InviteUserModal v-model:open="isInviteModalOpen" />
    <EditUserModal v-model:open="isEditModalOpen" :user="selectedUser" @save="handleSaveUser" />
    <ChangeRoleModal v-model:open="isRoleModalOpen" :user="selectedUser" @save="handleSaveRole" />

    <ConfirmationModal
      v-model:open="isDeleteModalOpen"
      title="Delete User?"
      :message="selectedUser ? `Are you sure you want to delete ${selectedUser.username}? All data, settings, and access permissions will be permanently deleted.` : ''"
      confirmLabel="Delete User"
      cancelLabel="Cancel"
      variant="error"
      :onConfirm="confirmDelete"
    />
  </div>
</template>
