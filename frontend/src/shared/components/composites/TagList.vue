<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue"

const props = withDefaults(
  defineProps<{
    tags: string[];
    maxLines?: number;
    maxTags?: number;
  }>(),
  {
    maxLines: 2,
    maxTags: Infinity,
  }
);

const container = ref<HTMLElement | null>(null);
const tagContainers = ref<HTMLElement[]>([]);
const badgeContainer = ref<HTMLElement | null>(null);

const visibleCount = ref(props.tags.length);
const hiddenCount = computed(() => props.tags.length - visibleCount.value);
const hiddenTags = computed(() => props.tags.slice(visibleCount.value));

// Count lines by comparing vertical position.
const getLineCount = () => {
  const visible = tagContainers.value.slice(0, visibleCount.value).filter(Boolean);
  if (hiddenCount.value > 0 && badgeContainer.value) visible.push(badgeContainer.value);
  if (!visible.length) return 0;

  let lines = 1;
  let lastTop = visible[0]?.offsetTop ?? 0;

  for (const tagContainer of visible) {
    if (tagContainer.offsetTop > lastTop + 4) { // A bit of tolerance.
      lines++;
      lastTop = tagContainer.offsetTop;
    }
  }
  return lines;
}

// If too many lines are used -> decrease shown elements.
const update = async () => {
  await nextTick();
  if (!container.value || !tagContainers.value.length) return;

  visibleCount.value = Math.min(props.tags.length, props.maxTags);
  await nextTick();

  while (visibleCount.value > 0 && getLineCount() > props.maxLines) {
    visibleCount.value--;
    await nextTick();
  }
}

// When the tags change, it needs to recalculate the amount of tags that fit.
watch(() => [props.tags, props.maxLines, props.maxTags], update);

// When the surrounding box changes size, it needs to recalculate the amount of tags that fit.
let observer: ResizeObserver | null = null;
onMounted(() => {
  update();
  if (container.value?.parentElement) {
    observer = new ResizeObserver(update);
    observer.observe(container.value.parentElement);
  }
})
onBeforeUnmount(() => observer?.disconnect());
</script>

<template>
  <div ref="container" class="row wrap no-padding no-margin tag-container">
    <template v-for="tag in tags" :key="tag">
      <div ref="tagContainers" :style="{ display: hiddenTags.includes(tag) ? 'none' : 'inline-flex' }">
        <slot name="tag" :tag="tag">{{ tag }}</slot>
      </div>
    </template>

    <div ref="badgeContainer" :style="{ display: hiddenCount > 0 ? 'inline-flex' : 'none' }">
      <FloatingElement hover align="left">
        <template #button>
          <slot name="badge" :count="hiddenCount">+{{ hiddenCount }}</slot>
        </template>
        <template #menu>
          <div class="no-round active surface-container-highest shadow left-align more-tag-container">
            <template v-for="tag in hiddenTags" :key="tag">
              <div style="display: inline-flex;">
                <slot name="tag" :tag="tag">{{ tag }}</slot>
              </div>
            </template>
          </div>
        </template>
      </FloatingElement>
    </div>
  </div>
</template>

<style scoped>
.tag-container, .more-tag-container {
    gap: 8px;
}
.more-tag-container {
    display: flex;
    flex-direction: column;
    padding: 12px;
}
</style>
