<script setup lang="ts">
import { ref, computed, watch } from "vue";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";
import CodeOutputField from "@/shared/components/composites/CodeOutputField.vue";
import { useUserManagementStore } from "@/features/settings/stores/userManagementStore";
import { formatExpiryDuration } from "@/shared/utils/formatters";

const open = defineModel<boolean>("open", { default: false });

const usersManagementStore = useUserManagementStore();

const registrationToken = ref("");
const expiresMinutes = ref<number | null>(null);
const isLoading = ref(false);
const errorMessage = ref("");

watch(open, async (isOpen) => {
  if (isOpen) {
    registrationToken.value = "";
    expiresMinutes.value = null;
    errorMessage.value = "";
    await createJoinToken();
  }
});

async function createJoinToken() {
  isLoading.value = true;
  errorMessage.value = "";
  try {
    const res = await usersManagementStore.generateInviteToken();
    if (res.success && res.data) {
      registrationToken.value = res.data.registrationToken;
      expiresMinutes.value = res.data.expires;
    } else {
      errorMessage.value = res.error || "Failed to generate invite token.";
    }
  } catch (error) {
    errorMessage.value = "Failed to generate invite token.";
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
        <h5 class="bold dialog-title no-margin">Invite User</h5>
      </div>
    </template>

    <div class="modal-body-content">
      <div v-if="errorMessage" class="chip max no-round error-text grid bottom-margin error-box">
        <i class="s1">error</i>
        <span class="s11 wrap text-wrap">{{ errorMessage }}</span>
      </div>

      <div v-if="isLoading" class="center-align padding">
        <progress class="circle"></progress>
      </div>

      <template v-else-if="registrationToken">
        <p class="on-surface no-margin">
          Share this registration token with the invited user:
        </p>

        <CodeOutputField
          :value="registrationToken"
          label="token"
          :show-share="false"
        />

        <div v-if="expiresMinutes" class="row no-margin middle-align expiry-row">
          <i class="small text-secondary">schedule</i>
          <span class="small-text text-secondary">{{ formattedExpiry }}</span>
        </div>
      </template>
    </div>
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
  gap: 0.5rem;
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
