<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import ScrollableContainer from "@/components/general/ScrollableContainer.vue";

const props = withDefaults(
  defineProps<{
    tags: string[];
    selectedTags: string[];
    alignment?: "center" | "flex-start";
  }>(),
  {
    alignment: "center",
  }
);

const emit = defineEmits<{
  (e: "toggle-tag", tag: string): void;
}>();

const scrollContainer = ref<InstanceType<typeof ScrollableContainer> | null>(null);

function triggerCheckScroll() {
  scrollContainer.value?.triggerCheckScroll();
}

// Trigger recalculation of arrow buttons on tag list.
watch(
  () => [props.tags, props.selectedTags],
  async () => {
    await nextTick();
    triggerCheckScroll();
  },
  { deep: true }
);

defineExpose({ triggerCheckScroll });
</script>

<template>
  <ScrollableContainer ref="scrollContainer" :content-alignment="alignment">
    <button
      v-for="tag in tags"
      :key="tag"
      class="chip medium no-border no-round tag"
      :class="{
        primary: selectedTags.includes(tag),
        'surface-container': !selectedTags.includes(tag),
      }"
      @click="emit('toggle-tag', tag)"
    >
      {{ tag }}
    </button>
  </ScrollableContainer>
</template>