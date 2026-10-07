<script setup lang="ts">
import type { SettingItem } from "@/components/settings/settingsSchema";
import type { SettingsInstance } from "@/composables/settings";
import { getItemValue, setItemValue } from "@/components/settings/settingsUtils";

const props = defineProps<{
  item: SettingItem;
  disabled: boolean;
  instance: SettingsInstance<any>;
}>();

function onChange(e: Event) {
  const target = e.target as HTMLInputElement;
  setItemValue(props.item, target.checked, props.instance);
}
</script>

<template>
  <label class="switch" style="margin: 0;" @click.stop>
    <input
      type="checkbox"
      :checked="Boolean(getItemValue(item, props.instance))"
      :disabled="disabled"
      @change="onChange"
    />
    <span></span>
  </label>
</template>