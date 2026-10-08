<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import ScrollableContainer from "@/shared/components/primitives/ScrollableContainer.vue";
import { useProjectsQuery } from "@/features/projects/composables/projectsQuery";

withDefaults(
  defineProps<{
    alignment?: "center" | "flex-start";
  }>(),
  {
    alignment: "center",
  }
);

const projectsQuery = useProjectsQuery();

const scrollContainer = ref<InstanceType<typeof ScrollableContainer> | null>(null);

function triggerCheckScroll() {
  scrollContainer.value?.triggerCheckScroll();
}

// Trigger recalculation of arrow buttons on tag list.
watch(
  () => [projectsQuery.availableTags, projectsQuery.selectedTagFilters],
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
      v-for="tag in projectsQuery.availableTags.value"
      :key="tag"
      class="chip medium no-border no-round tag"
      :class="{
        primary: projectsQuery.selectedTagFilters.value.includes(tag),
        'surface-container': !projectsQuery.selectedTagFilters.value.includes(tag),
      }"
      @click="projectsQuery.toggleTagFilter(tag)"
    >
      {{ tag }}
    </button>
  </ScrollableContainer>
</template>
