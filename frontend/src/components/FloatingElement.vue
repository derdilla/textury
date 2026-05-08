<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';

// The popup is currently always placed below the button to trigger it.
// Hover is for tooltips. Align can be any of 'left', 'center', 'right'.
const props = defineProps({
  hover: Boolean,
  align: {
    type: String,
    default: 'center'
  }
});

// Send out events when popup is shown or removed.
const emit = defineEmits(['toggle']);

const isOpen = ref(false), container = ref(null), pos = ref({});

const isContextMenu = ref(false);

const update = (e = null) => {
  if (!isOpen.value || !container.value) return;
  const r = container.value.getBoundingClientRect();
  let x, y, shift;

  // Use mouse coordinates if openAt was used
  if (isContextMenu.value && e) {
    x = e.clientX;
    y = e.clientY;
    shift = '0%'; // Align top-left to cursor
  } else {
    y = r.bottom + 8;
    if (props.align === 'left') {
      x = r.right;
    } else if (props.align === 'right') {
      x = r.left;
    } else {
      x = r.left + (r.width / 2);
    }
    shift = null;
  }

  pos.value = {
    position: 'fixed',
    zIndex: 999999,
    top: `${y}px`,
    left: `${x}px`,
    '--x-override': shift
  };
};

const lastEvent = ref(null);

// Open the popup at the mouse position.
const openAt = (e) => {
  isContextMenu.value = true;
  lastEvent.value = e;
  isOpen.value = true;
};

const toggle = () => {
  if (props.hover || container.value?.querySelector('.is-disabled')) return;
  isContextMenu.value = false;
  isOpen.value = !isOpen.value;
};

const setHover = (val) => {
  if (!props.hover) return;
  isContextMenu.value = false;
  isOpen.value = val;
};

watch(isOpen, (val) => {
  if (val) {
    update(lastEvent.value);
  } else {
    isContextMenu.value = false;
    lastEvent.value = null;
  }
  emit('toggle', val);
});

// When the button moves, it needs to recalculate the position of the popup.
onMounted(() => {
  window.addEventListener('resize', () => update(lastEvent.value));
  window.addEventListener('scroll', () => update(lastEvent.value), true);
});
onUnmounted(() => {
  window.removeEventListener('resize', () => update(lastEvent.value));
  window.removeEventListener('scroll', () => update(lastEvent.value), true);
});

defineExpose({ openAt, isOpen });
</script>

<template>
  <div style="display: inline-block" ref="container" @mouseenter="setHover(true)" @mouseleave="setHover(false)">
    <div @click.stop="toggle"><slot name="button" /></div>

    <Teleport to="body">
      <div v-if="isOpen && !hover" class="shield" @click.stop="isOpen = false" />

      <Transition name="pop">
        <div v-if="isOpen" class="wrapper" :class="`is-aligned-${align}`" :style="pos" @click="isOpen = false">
          <slot name="menu" />
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.shield {
  position: fixed;
  inset: 0;
  z-index: 999998;
}

.wrapper {
  position: fixed;
  --x-shift: translateX(-50%);
  transform: var(--x-override, var(--x-shift));
}
.wrapper.is-aligned-left {
  --x-shift: translateX(-100%);
}
.wrapper.is-aligned-center {
  --x-shift: translateX(-50%);
}
.wrapper.is-aligned-right {
  --x-shift: translateX(0%);
}

.wrapper :deep(menu),
.wrapper :deep(.tooltip),
.wrapper :deep(menu)::before,
.wrapper :deep(menu)::after {
  visibility: visible !important;
  position: static !important;
  transform: none !important;
  opacity: 1 !important;
}

.pop-enter-active, .pop-leave-active {
  transition: opacity 0.1s ease, transform 0.1s ease;
}
.pop-enter-from, .pop-leave-to {
  opacity: 0;
  transform: var(--x-shift) scale(0.9);
}
</style>
