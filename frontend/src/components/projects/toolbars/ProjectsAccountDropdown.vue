<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import FloatingElement from "@/components/general/FloatingElement.vue";
import { editCtrl } from "@/services/editorController";
import { invite, logout } from "@/services/api/managementApi";

const router = useRouter();

const inviteDialog = ref<HTMLDialogElement>();
const registrationToken = ref("");

async function createJoinToken() {
  const success = await invite();
  if (success) {
    registrationToken.value = success.registration_token;
    inviteDialog.value?.showPopover();
  }
}

async function handleLogout() {
  await logout();
  editCtrl.user = null;
  router.push("/login");
}

function openSettings() {
  router.push("/settings");
}
</script>

<template>
  <div class="account-menu-wrapper">
    <FloatingElement align="right">
      <template #button>
        <button tabindex="0" class="no-round chip medium secondary-container no-border">
          <i>person</i>
          <span>{{ editCtrl.user?.username }}</span>
        </button>
      </template>
      <template #menu>
        <menu class="dropdown no-wrap no-round active surface-container">
          <button class="transparent no-round left-align" @click="openSettings">
            <i class="small">settings</i> Settings
          </button>
          <button
            v-if="editCtrl.user?.is_admin"
            class="transparent no-round left-align"
            @click="createJoinToken"
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

    <!-- Registration Token Dialog -->
    <dialog ref="inviteDialog" popover>
      <p>Tell the invited person to use registration token:</p>
      <pre>{{ registrationToken }}</pre>
    </dialog>
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