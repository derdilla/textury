<script setup lang="ts">
import { ref, computed } from "vue";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";

const props = withDefaults(
  defineProps<{
    url: string;
    title?: string;
    text?: string;
    label?: string;
    disabled?: boolean;
  }>(),
  {
    title: "Share",
    text: "Here is the link:",
    label: "URL",
    disabled: false,
  }
);

const isShared = ref(false);

const canNativeShare = computed(() => typeof window !== "undefined" && !!window.navigator?.share);

async function handleNativeShare() {
  if (!props.url || props.disabled || !window.navigator.share) return;

  try {
    await window.navigator.share({ title: props.title, text: props.text, url: props.url });
    isShared.value = true;
    setTimeout(() => (isShared.value = false), 2000);
  } catch (error: any) {
    if (error?.name !== "AbortError") {
      console.error("Error sharing:", error);
    }
  }
}
</script>

<template>
  <FloatingElement v-if="canNativeShare" :hover="!disabled" align="left">
    <template #button>
      <button class="no-round border square" :disabled="disabled" @click="handleNativeShare">
        <i>{{ isShared ? 'check' : 'share' }}</i>
      </button>
    </template>
    <template v-if="!disabled" #menu>
      <span class="tooltip bottom active">{{ isShared ? 'Shared!' : `Share ${label}` }}</span>
    </template>
  </FloatingElement>
</template>
