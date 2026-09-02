<script setup lang="ts">
import { watch, onUnmounted } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    title?: string;
    closeOnBackdrop?: boolean;
    closeOnEsc?: boolean;
    showClose?: boolean;
    maxWidth?: string;
    responsive?: boolean;
  }>(),
  {
    closeOnBackdrop: true,
    closeOnEsc: true,
    showClose: true,
    maxWidth: "500px",
    responsive: true,
  }
);

const emit = defineEmits<{
  (e: "update:modelValue", value: boolean): void;
  (e: "close"): void;
}>();

const close = () => {
  emit("update:modelValue", false);
  emit("close");
};

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === "Escape" && props.closeOnEsc) close();
};

watch(
  () => props.modelValue,
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
      <div v-if="modelValue" class="popover-backdrop" @click.self="closeOnBackdrop && close()">
        <div class="popover-dialog surface-container shadow active" :class="{ responsive }" :style="{ maxWidth }">

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
  background-color: rgba(0, 0, 0, 0.6);
  padding: 1rem;
  box-sizing: border-box;
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
  border-radius: 12px;
  padding: 1.25rem;
  box-sizing: border-box;
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
</style>