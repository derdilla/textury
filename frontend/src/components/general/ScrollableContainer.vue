<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick, watch } from "vue";

const props = withDefaults(
  defineProps<{
    scrollStep?: number;
    contentAlignment?: 'center' | 'flex-start';
  }>(),
  {
    scrollStep: 200,
    contentAlignment: 'center',
  }
);

const scrollContainer = ref<HTMLElement | null>(null);
const canScrollLeft = ref(false);
const canScrollRight = ref(false);

function checkScrollState() {
  const el = scrollContainer.value;
  if (!el || el.clientWidth === 0) {
    canScrollLeft.value = false;
    canScrollRight.value = false;
    return;
  }
  canScrollLeft.value = el.scrollLeft > 1; // A bit of tolerance.
  canScrollRight.value = el.scrollWidth - el.clientWidth - el.scrollLeft > 1; // A bit of tolerance.
}

function triggerCheckScroll() {
  requestAnimationFrame(() => {
    checkScrollState();
  });
}

function scroll(direction: 'left' | 'right') {
  const el = scrollContainer.value;
  if (!el) return;
  const offset = direction === 'left' ? -props.scrollStep : props.scrollStep;
  el.scrollBy({ left: offset, behavior: 'smooth' });
}

let resizeObserver: ResizeObserver | null = null;

function setupResizeObserver() {
  if (typeof window === "undefined" || !window.ResizeObserver) return;

  if (!resizeObserver) {
    resizeObserver = new ResizeObserver(() => triggerCheckScroll());
  } else {
    resizeObserver.disconnect();
  }

  if (scrollContainer.value) {
    resizeObserver.observe(scrollContainer.value);
    if (scrollContainer.value.parentElement) {
      resizeObserver.observe(scrollContainer.value.parentElement);
    }
  }
}

watch(scrollContainer, async () => {
  await nextTick();
  setupResizeObserver();
  triggerCheckScroll();
});

onMounted(() => {
  triggerCheckScroll();
  window.addEventListener('resize', triggerCheckScroll);
  if (document.fonts) {
    document.fonts.ready.then(triggerCheckScroll);
  }
});
onBeforeUnmount(() => {
  window.removeEventListener('resize', triggerCheckScroll);
  if (resizeObserver) resizeObserver.disconnect();
});

defineExpose({ triggerCheckScroll });
</script>

<template>
  <div class="tag-wrapper">
    <!--Left Scroll Arrow-->
    <button @click="scroll('left')" class="square border scroll-button left-button" :class="{ 'is-hidden': !canScrollLeft }" tabindex="-1">
      <i>chevron_left</i>
    </button>

    <!--Scroll Container-->
    <div ref="scrollContainer" @scroll="checkScrollState" class="row scrollable-buttons small-scroll-bar max hide-scrollbar"
      :class="[ contentAlignment === 'center' ? 'align-center' : 'align-start',
        { 'mask-left': canScrollLeft && !canScrollRight, 'mask-right': !canScrollLeft && canScrollRight, 'mask-both': canScrollLeft && canScrollRight }]">
      <slot />
    </div>

    <!--Right Scroll Arrow-->
    <button @click="scroll('right')" class="square border scroll-button right-button" :class="{ 'is-hidden': !canScrollRight }" tabindex="-1">
      <i>chevron_right</i>
    </button>
  </div>
</template>

<style scoped>
.tag-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  min-width: 0;
  flex-grow: 1;
  flex-shrink: 2 !important;
  height: 2.5rem;
  box-sizing: border-box;
}

.scroll-button {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  z-index: 2;
  flex-shrink: 0;
  margin: 0 !important;
  transition: opacity 0.2s ease;
}

.scroll-button.left-button {
  left: 0;
}

.scroll-button.right-button {
  right: 0;
}

.scroll-button.is-hidden {
  opacity: 0 !important;
  visibility: hidden !important;
  pointer-events: none !important;
}

.scrollable-buttons {
  width: 100%;
  flex-grow: 1;
  flex-shrink: 1;
  overflow-x: auto;
  overflow-y: hidden !important;
  flex-wrap: nowrap !important;
  display: flex;
  align-items: center;
  scroll-behavior: smooth;
  padding: 0;
  margin: 0 !important;
  height: 2.5rem;
  box-sizing: border-box;
}

.align-center {
  justify-content: safe center !important;
}

.align-start {
  justify-content: flex-start !important;
}

.hide-scrollbar {
  scrollbar-width: none;
  -ms-overflow-style: none;
}
.hide-scrollbar::-webkit-scrollbar {
  display: none;
}

.mask-left {
  mask-image: linear-gradient(to right, transparent 0px, transparent 35px, black 75px, black 100%);
  -webkit-mask-image: linear-gradient(to right, transparent 0px, transparent 35px, black 75px, black 100%);
}
.mask-right {
  mask-image: linear-gradient(to right, black 0%, black calc(100% - 75px), transparent calc(100% - 35px), transparent 100%);
  -webkit-mask-image: linear-gradient(to right, black 0%, black calc(100% - 75px), transparent calc(100% - 35px), transparent 100%);
}
.mask-both {
  mask-image: linear-gradient(to right, transparent 0px, transparent 35px, black 75px, black calc(100% - 75px), transparent calc(100% - 35px), transparent 100%);
  -webkit-mask-image: linear-gradient(to right, transparent 0px, transparent 35px, black 75px, black calc(100% - 75px), transparent calc(100% - 35px), transparent 100%);
}
</style>