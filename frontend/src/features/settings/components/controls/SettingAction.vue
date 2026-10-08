<script setup lang="ts">
import { ref, computed } from "vue";
import type { SettingItem } from "@/features/settings/utils/settingsSchema";
import type { SettingsInstance } from "@/features/settings/composables/settings";

const props = defineProps<{
  item: SettingItem;
  disabled?: boolean;
  instance: SettingsInstance<any>;
}>();

const isModalOpen = ref(false);

const buttonColorClass = computed(() => {
  if (props.item.destructive) return "error";
  return props.item.important ? "secondary-container" : "surface-container-highest";
});

const resolvedModalProps = computed(() => {
  if (typeof props.item.modalProps === "function") {
    return props.item.modalProps(props.instance);
  }
  return props.item.modalProps ?? {};
});

function executeAction(e?: MouseEvent) {
  if (e) e.stopPropagation();
  if (props.disabled) return;

  if (props.item.modalComponent) {
    isModalOpen.value = true;
  }

  props.item.onClick?.(props.instance);
}

defineExpose({
  executeAction
});
</script>

<template>
  <div class="setting-action-wrapper" @click.stop>
    <!-- Button Style Action -->
    <button
      v-if="item.actionStyle === 'button'"
      class="no-round"
      :class="[
        buttonColorClass,
        { disabled }
      ]"
      :disabled="disabled"
      @click="executeAction"
    >
      <span>{{ item.actionLabel ?? item.title }}</span>
      <i v-if="item.suffixIcon" class="small">{{ item.suffixIcon }}</i>
    </button>

    <!-- Default Icon Style Action -->
    <i
      v-else-if="item.suffixIcon !== null && item.suffixIcon !== ''"
      class="action-icon"
      :class="{ 'disabled-icon': disabled }"
      @click="executeAction"
    >
      {{ item.suffixIcon ?? 'open_in_new' }}
    </i>

    <!-- Modal Container -->
    <component
      :is="item.modalComponent"
      v-if="item.modalComponent"
      v-model:open="isModalOpen"
      v-bind="resolvedModalProps"
    />
  </div>
</template>

<style scoped>
.setting-action-wrapper {
  display: inline-flex;
  align-items: center;
}

.disabled-icon {
  opacity: 0.38;
  cursor: not-allowed;
}

button.no-round {
  margin: 0;
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
}
</style>
