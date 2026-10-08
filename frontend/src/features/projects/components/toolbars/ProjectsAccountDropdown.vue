<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";
import { api } from "@/api/api";
import { useAuthStore } from "@/shared/stores/authStore";

const router = useRouter();
const authStore = useAuthStore();

const inviteDialog = ref<HTMLDialogElement>();
const registrationToken = ref("");
const expiresMinutes = ref<number | null>(null);

async function createJoinToken() {
  try {
    const res = await api.POST("/settings/users/invite");
    if (!res.error && res.data) {
      registrationToken.value = res.data.registrationToken;
      expiresMinutes.value = res.data.expires;
      inviteDialog.value?.showPopover();
    }
  } catch (error) {
    console.error("Failed to generate invite token:", error);
  }
}

async function handleLogout() {
  await authStore.logout();
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
      <p v-if="expiresMinutes" class="small-text">
        Expires in {{ expiresMinutes }} minutes.
      </p>
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
