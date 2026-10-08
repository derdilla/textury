<script setup lang="ts">
import type { SettingItem } from "@/features/settings/utils/settingsSchema";
import type { SettingsInstance } from "@/features/settings/composables/settings";
import { getItemValue, setItemValue } from "@/features/settings/utils/settingsUtils";

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
