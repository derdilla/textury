<script setup lang="ts">
import { invite } from '@/services/api/managementApi';
import { ref } from 'vue';


const dialog = ref<HTMLDialogElement>();

const registrationToken = ref("")

async function createJoinToken() {
  const success = await invite();
  if (success) {
    registrationToken.value = success.registration_token;
    dialog.value?.showPopover();
  }
}

</script>
<template>
<button class="border circle" @click="createJoinToken">
  <i>person_add</i>
  <span class="tooltip bottom">Invite user</span>
</button>
<dialog
    popover
    ref="dialog">
    <p>Tell the invited person to use registration token:</p>
    <pre>{{ registrationToken }}</pre>
  </dialog>
</template>
