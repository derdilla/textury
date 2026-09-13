<script setup lang="ts">
import type { SettingItem } from "@/components/settings/settingsSchema";
import { getItemValue, setItemValue } from "@/components/settings/settingsUtils";
import type { SettingsStore } from "@/config/settingsEngine";

const props = defineProps<{
  item: SettingItem;
  disabled: boolean;
  isFirstInGroup?: boolean;
  store: SettingsStore<any>;
}>();

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  setItemValue(props.item, Number(target.value), props.store);
}
</script>

<template>
  <div class="slider no-margin" style="width: 160px; flex-shrink: 0;">
    <input
      type="range"
      :min="item.min ?? 0"
      :max="item.max ?? 100"
      :step="item.step"
      :value="getItemValue(item, props.store) ?? item.min ?? 0"
      :disabled="disabled"
      @input="onInput"
    />
    <span></span>
    <span class="tooltip" :class="{ bottom: isFirstInGroup }"></span>
  </div>
</template>