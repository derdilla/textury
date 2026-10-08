<script setup lang="ts">
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";

defineProps<{
  reason: string;
}>();
</script>

<template>
  <FloatingElement hover align="center" class="disabled-help-wrapper">
    <template #button>
      <button
        type="button"
        class="square no-round transparent medium disabled-help-btn"
        @click.stop
      >
        <i>help_outline</i>
      </button>
    </template>
    <template #menu>
      <span class="tooltip bottom">{{ reason }}</span>
    </template>
  </FloatingElement>
</template>

<style scoped>
.disabled-help-wrapper {
  margin-right: -0.25rem;
}
.disabled-help-btn {
  cursor: help;
}
</style>
