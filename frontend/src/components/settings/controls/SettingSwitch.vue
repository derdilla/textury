<script setup lang="ts">
import type { SettingItem } from "@/components/settings/settingsSchema";
import { getItemValue, setItemValue } from "@/components/settings/settingsUtils";
import type { SettingsStore } from "@/config/settingsEngine";

const props = defineProps<{
  item: SettingItem;
  disabled: boolean;
  store: SettingsStore<any>;
}>();

function onChange(e: Event) {
  const target = e.target as HTMLInputElement;
  setItemValue(props.item, target.checked, props.store);
}
</script>

<template>
  <label class="switch" style="margin: 0;" @click.stop>
    <input
      type="checkbox"
      :checked="Boolean(getItemValue(item, props.store))"
      :disabled="disabled"
      @change="onChange"
    />
    <span></span>
  </label>
</template>