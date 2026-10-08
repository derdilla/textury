<script setup lang="ts">
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";

const open = defineModel<boolean>("open", { default: false });

defineProps<{
  title?: string;
  message?: string;
}>();
</script>

<template>
  <ModalPopover v-model:open="open" :title="title ?? 'Modal'">
    <p class="no-margin-top">{{ message ?? "Modal content" }}</p>

    <template #actions>
      <button class="border no-round" @click="open = false">
        Close
      </button>
    </template>
  </ModalPopover>
</template>
