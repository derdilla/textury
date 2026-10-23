<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";
import InviteUserModal from "@/features/settings/components/custom/modals/InviteUserModal.vue";
import { useAuthStore } from "@/shared/stores/authStore";

const router = useRouter();
const authStore = useAuthStore();

const isInviteModalOpen = ref(false);
const floatingElement = ref();

function closeMenu() {
  if (floatingElement.value) {
    floatingElement.value.isOpen = false;
  }
}

function openInviteModal() {
  closeMenu();
  isInviteModalOpen.value = true;
}

async function handleLogout() {
  closeMenu();
  await authStore.logout();
}

function openSettings() {
  closeMenu();
  router.push("/settings");
}
</script>

<template>
  <div class="account-menu-wrapper">
    <FloatingElement ref="floatingElement" align="right">
      <template #button>
        <button tabindex="0" class="no-round chip medium secondary-container no-border">
          <i>person</i>
          <span>{{ authStore.state.user?.username ?? 'Not logged in' }}</span>
        </button>
      </template>
      <template #menu>
        <menu class="dropdown no-wrap no-round active surface-container">
          <button class="transparent no-round left-align" @click="openSettings">
            <i class="small">settings</i> Settings
          </button>
          <button
            v-if="authStore.state.user?.isAdmin ?? false"
            class="transparent no-round left-align"
            @click="openInviteModal"
          >
            <i class="small">person_add</i> Invite user
          </button>
          <li><hr /></li>
          <button class="transparent no-round left-align error-text" @click="handleLogout">
            <i class="small">logout</i> Log out
          </button>
        </menu>
      </template>
    </FloatingElement>

    <InviteUserModal v-model:open="isInviteModalOpen" />
  </div>
</template>

<style scoped>
.dropdown hr {
  margin: 3px;
}
.dropdown i {
  margin-right: 0.5rem;
}
.dropdown {
  gap: 4px;
}
.dropdown {
  box-shadow: none !important;
}
</style>
