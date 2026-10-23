<script setup lang="ts">
import { ref } from "vue";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";

const props = withDefaults(
  defineProps<{
    value: string;
    label?: string;
    disabled?: boolean;
  }>(),
  {
    label: "item",
    disabled: false,
  }
);

const isCopied = ref(false);

async function copyText() {
  if (!props.value || props.disabled) return;
  await navigator.clipboard.writeText(props.value);
  isCopied.value = true;
  setTimeout(() => {
    isCopied.value = false;
  }, 2000);
}
</script>

<template>
  <FloatingElement :hover="!disabled" align="left">
    <template #button>
      <button
        class="no-round border square"
        :disabled="disabled"
        @click="copyText"
      >
        <i>{{ isCopied ? 'check' : 'content_copy' }}</i>
      </button>
    </template>
    <template v-if="!disabled" #menu>
      <span class="tooltip bottom active">{{ isCopied ? 'Copied!' : `Copy ${label}` }}</span>
    </template>
  </FloatingElement>
</template>
