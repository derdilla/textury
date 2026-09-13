<script setup lang="ts">
import { computed } from "vue";
import type { SettingItem } from "@/components/settings/settingsSchema";
import { getItemValue, setItemValue } from "@/components/settings/settingsUtils";
import type { SettingsStore } from "@/config/settingsEngine";

const props = defineProps<{
  item: SettingItem;
  disabled: boolean;
  isFirstInGroup?: boolean;
  store: SettingsStore<any>;
}>();

const rangeValues = computed<[number, number]>(() => {
  const val = getItemValue(props.item, props.store);
  if (Array.isArray(val) && val.length === 2) {
    return val as [number, number];
  }
  return [props.item.min ?? 0, props.item.max ?? 100];
});

function onMinInput(e: Event) {
  const newMin = Number((e.target as HTMLInputElement).value);
  setItemValue(props.item, [newMin, rangeValues.value[1]], props.store);
}

function onMaxInput(e: Event) {
  const newMax = Number((e.target as HTMLInputElement).value);
  setItemValue(props.item, [rangeValues.value[0], newMax], props.store);
}
</script>

<template>
  <div class="slider no-margin" style="width: 160px; flex-shrink: 0;">
    <input
      type="range"
      :min="item.min ?? 0"
      :max="item.max ?? 100"
      :step="item.step"
      :value="rangeValues[0]"
      :disabled="disabled"
      @input="onMinInput"
    />
    <input
      type="range"
      :min="item.min ?? 0"
      :max="item.max ?? 100"
      :step="item.step"
      :value="rangeValues[1]"
      :disabled="disabled"
      @input="onMaxInput"
    />
    <span></span>
    <span class="tooltip" :class="{ bottom: isFirstInGroup }"></span>
    <span class="tooltip" :class="{ bottom: isFirstInGroup }"></span>
  </div>
</template>