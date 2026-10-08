<script setup lang="ts">
import type { SettingItem } from "@/features/settings/utils/settingsSchema";
import type { SettingsInstance } from "@/features/settings/composables/settings";
import { getItemValue, setItemValue } from "@/features/settings/utils/settingsUtils";

const props = defineProps<{
  item: SettingItem;
  disabled: boolean;
  isFirstInGroup?: boolean;
  instance: SettingsInstance<any>;
}>();

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  setItemValue(props.item, Number(target.value), props.instance);
}
</script>

<template>
  <div class="slider no-margin" style="width: 160px; flex-shrink: 0;">
    <input
      type="range"
      :min="item.min ?? 0"
      :max="item.max ?? 100"
      :step="item.step"
      :value="getItemValue(item, props.instance) ?? item.min ?? 0"
      :disabled="disabled"
      @input="onInput"
    />
    <span></span>
    <span class="tooltip" :class="{ bottom: isFirstInGroup }"></span>
  </div>
</template>
