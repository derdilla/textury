<script setup lang="ts">
import CopyButton from "@/shared/components/primitives/CopyButton.vue";
import ShareButton from "@/shared/components/primitives/ShareButton.vue";

withDefaults(
  defineProps<{
    value: string;
    label?: string;
    shareTitle?: string;
    shareText?: string;
    showShare?: boolean;
  }>(),
  {
    label: "URL",
    showShare: true,
  }
);
</script>

<template>
  <div class="row no-margin middle-align token-container">
    <div class="max surface-container-highest token-box no-round horizontal-padding">
      <span class="token-text">{{ value }}</span>
    </div>
    <CopyButton :value="value" :label="label" />
    <ShareButton
      v-if="showShare"
      :url="value"
      :title="shareTitle"
      :text="shareText"
      :label="label"
    />
  </div>
</template>

<style scoped>
.token-container {
  gap: 0.75rem;
  width: 100%;
}

.token-box {
  height: 2.5rem;
  line-height: 2.5rem;
  box-sizing: border-box;
  overflow-x: auto;
  overflow-y: hidden;
  user-select: all;
  white-space: nowrap;
}
</style>
