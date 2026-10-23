<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";
import ConfirmationModal from "@/shared/components/composites/ConfirmationModal.vue";
import InviteUserModal from "../modals/InviteUserModal.vue";
import EditUserModal from "../modals/EditUserModal.vue";
import ChangeRoleModal from "../modals/ChangeRoleModal.vue";
import { useUserManagementStore } from "@/features/settings/stores/userManagementStore";
import type { PrivateAccountInfo, ProjectId, UserId } from "@/api/types";

defineProps<{
  projectId?: ProjectId;
}>();

const usersManagementStore = useUserManagementStore();

const users = computed<PrivateAccountInfo[]>(() => usersManagementStore.users.value);

const floatingElements = ref<Record<UserId, any>>({});
const selectedUser = ref<PrivateAccountInfo | null>(null);
const menuOpenStates = ref<Record<UserId, boolean>>({});

const isInviteModalOpen = ref(false);
const isEditModalOpen = ref(false);
const isRoleModalOpen = ref(false);
const isDeleteModalOpen = ref(false);

onMounted(async () => {
  if (!usersManagementStore.state.isInitialized) {
    await usersManagementStore.fetchUsers();
  }
});

const closeMenu = (userId: UserId) => {
  if (floatingElements.value[userId]) {
    floatingElements.value[userId].isOpen = false;
  }
};

function handleEdit(user: PrivateAccountInfo) {
  closeMenu(user.id);
  selectedUser.value = user;
  isEditModalOpen.value = true;
}

function handleChangeRole(user: PrivateAccountInfo) {
  closeMenu(user.id);
  selectedUser.value = user;
  isRoleModalOpen.value = true;
}

function handleDelete(user: PrivateAccountInfo) {
  closeMenu(user.id);
  selectedUser.value = user;
  isDeleteModalOpen.value = true;
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
    <!-- Header -->
    <div class="row padding middle-align no-margin bottom-margin">
      <div class="max">
        <h6 class="bold no-margin">Users</h6>
        <div class="small-text text-secondary">Manage accounts, roles, and user projects.</div>
      </div>
      <button class="no-round primary" @click="isInviteModalOpen = true">
        <i>person_add</i>
        <span>Invite User</span>
      </button>
    </div>

    <!-- User List -->
    <div class="surface-container user-list-wrapper">
      <div v-if="usersManagementStore.state.isLoading && !users.length" class="center-align padding">
        <progress class="circle"></progress>
      </div>

      <div v-for="user in users" :key="user.id"
        class="row padding middle-align no-margin user-row border-top"
        :class="{ 'menu-active': menuOpenStates[user.id] }">
        <i>account_circle</i>

        <div class="max">
          <div>{{ user.username }}</div>
          <div class="small-text text-secondary">{{ user.email }}</div>
        </div>

        <span v-if="user.isAdmin" class="chip no-round no-border surface-container-highest no-margin role-chip"
          :class="{ 'menu-active': menuOpenStates[user.id] }">
          Admin
        </span>

        <FloatingElement :ref="(element) => { if (element) floatingElements[user.id] = element; }"
          align="left" class="relative action-menu" @toggle="menuOpenStates[user.id] = $event">
          <template #button>
            <button tabindex="0" class="square transparent no-margin">
              <i>more_vert</i>
            </button>
          </template>
          <template #menu>
            <menu class="dropdown no-wrap no-round active surface-container">
              <button @click.stop="handleEdit(user)" class="transparent no-round left-align">
                <i class="small">edit</i> Edit User
              </button>
              <button @click.stop="handleChangeRole(user)" class="transparent no-round left-align">
                <i class="small">admin_panel_settings</i> Change Role
              </button>
              <!--
              <button @click.stop="closeMenu(user.id)" class="transparent no-round left-align">
                <i class="small">lock_reset</i> Reset Password
              </button>
              <li><hr /></li>
              <button @click.stop="closeMenu(user.id)" class="transparent no-round left-align">
                <i class="small">folder</i> View Projects
              </button>
              -->
              <li><hr /></li>
              <button @click.stop="handleDelete(user)" class="transparent no-round left-align error-text">
                <i class="small">delete</i> Delete
              </button>
            </menu>
          </template>
        </FloatingElement>
      </div>
    </div>

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

<style scoped>
.user-list-wrapper {
  border-radius: initial;
}
.user-row {
  transition: background-color 0.2s ease;
}
.user-row:hover {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 95%) !important;
}
.user-row.menu-active {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 90%) !important;
}
.role-chip {
  transition: background-color 0.2s ease;
}
.user-row:hover .role-chip {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 90%) !important;
}
.user-row.menu-active .role-chip {
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 90%) !important;
}
.border-top {
  border-top: 1px solid var(--outline-variant);
}
.text-secondary {
  opacity: 0.7;
  font-size: 0.85rem;
}
.action-menu {
  margin-left: -0.25rem;
}
.dropdown hr {
  margin: 3px;
}
.dropdown i {
  margin-right: 0.5rem;
}
.dropdown {
  gap: 4px;
  box-shadow: none !important;
}
</style>
