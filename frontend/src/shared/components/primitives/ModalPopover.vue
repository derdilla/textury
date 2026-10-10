<script setup lang="ts">
import { watch, onUnmounted } from "vue";

const open = defineModel<boolean>("open", { default: false });

const props = withDefaults(
  defineProps<{
    title?: string;
    closeOnBackdrop?: boolean;
    closeOnEsc?: boolean;
    showClose?: boolean;
    maxWidth?: string;
    responsive?: boolean;
    noPadding?: boolean;
  }>(),
  {
    closeOnBackdrop: true,
    closeOnEsc: true,
    showClose: true,
    maxWidth: "500px",
    responsive: true,
    noPadding: false,
  }
);

const emit = defineEmits<{
  (e: "close"): void;
}>();

const close = () => {
  open.value = false;
  emit("close");
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === "Escape" && props.closeOnEsc) close();
};

watch(
  open,
  (isOpen) => {
    if (typeof window === "undefined") return;
    document.body.style.overflow = isOpen ? "hidden" : "";
    if (isOpen) window.addEventListener("keydown", handleKeyDown);
    else window.removeEventListener("keydown", handleKeyDown);
  },
  { immediate: true }
);

onUnmounted(() => {
  if (typeof window !== "undefined") {
    document.body.style.overflow = "";
    window.removeEventListener("keydown", handleKeyDown);
  }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="popover-fade">
      <div v-if="open" class="popover-backdrop" @click.self="closeOnBackdrop && close()">
        <div
          class="popover-dialog surface-container active"
          :class="{ responsive, 'no-padding': noPadding }"
          :style="{ maxWidth }"
        >
          <button v-if="showClose" tabindex="0" @click="close" class="square transparent close-button">
            <i>close</i>
          </button>

          <nav v-if="title || $slots.header" class="popover-header">
            <slot name="header">
              <h6 class="max bold dialog-title">{{ title }}</h6>
            </slot>
          </nav>

          <div class="popover-content"><slot /></div>

          <nav v-if="$slots.actions" class="right-align popover-actions"><slot name="actions" /></nav>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.popover-backdrop {
  position: fixed;
  inset: 0;
  width: 100vw;
  height: 100vh;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: color-mix(in srgb, rgba(0, 0, 0, 0.3), var(--primary) 10%);
  padding: 1rem;
  box-sizing: border-box;
}

.dark .popover-backdrop {
  background-color: rgba(0, 0, 0, 0.6);
}

.popover-dialog {
  position: relative;
  margin: 0 auto;
  border: none;
  outline: none;
  width: 100%;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  z-index: 10000;
  border-radius: 18px;
  padding: 1.25rem;
  box-sizing: border-box;
}

.popover-dialog.no-padding {
  padding: 0;
  height: 85vh;
  max-height: 550px;
}

.popover-dialog.no-padding .popover-content {
  margin-top: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.close-button {
  position: absolute;
  top: 0.75rem;
  right: 0.75rem;
  z-index: 10;
}

.popover-header {
  display: flex;
  align-items: center;
  width: 100%;
  padding-right: 2.5rem;
  margin: 0 !important;
}

.dialog-title {
  margin: 0;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.popover-content {
  flex-grow: 1;
  margin-top: 1rem;
}

.popover-actions {
  margin-top: 1rem;
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}

@media (max-width: 600px) {
  .popover-backdrop {
    padding: 0;
  }

  .popover-dialog.responsive {
    width: 100vw !important;
    height: 100vh !important;
    max-width: 100vw !important;
    max-height: 100vh !important;
    border-radius: 0 !important;
  }
}

.popover-fade-enter-active,
.popover-fade-leave-active {
  transition: opacity 0.2s ease;
}

.popover-fade-enter-active .popover-dialog,
.popover-fade-leave-active .popover-dialog {
  transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.2s ease;
}

.popover-fade-enter-from,
.popover-fade-leave-to {
  opacity: 0;
}

.popover-fade-enter-from .popover-dialog,
.popover-fade-leave-to .popover-dialog {
  opacity: 0;
  transform: scale(0.95);
}

@media (prefers-reduced-transparency: reduce) {
  .popover-backdrop {
    background-color: color-mix(in srgb, var(--surface), #000000 10%) !important;
  }
}
</style>
