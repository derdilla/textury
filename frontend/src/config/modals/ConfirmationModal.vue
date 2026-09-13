<script setup lang="ts">
import ModalPopover from "@/components/general/ModalPopover.vue";

const open = defineModel<boolean>("open", { default: false });

const props = withDefaults(
  defineProps<{
    title?: string;
    message?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    variant?: string;
    onConfirm?: () => void;
  }>(),
  {
    title: "Confirm Action",
    confirmLabel: "Confirm",
    cancelLabel: "Cancel",
    variant: "primary",
  }
);

function handleConfirm() {
  if (props.onConfirm) {
    props.onConfirm();
  } else {
    alert(`Action executed: ${props.title}`);
  }
  open.value = false;
}
</script>

<template>
  <ModalPopover v-model:open="open" :title="title">
    <p v-if="message">{{ message }}</p>

    <template #actions>
      <div class="row gap">
        <button class="border no-round" @click="open = false">
          {{ cancelLabel }}
        </button>
        <button class="no-round" :class="variant" @click="handleConfirm">
          {{ confirmLabel }}
        </button>
      </div>
    </template>
  </ModalPopover>
</template>