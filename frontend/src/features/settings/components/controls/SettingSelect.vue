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
  const target = e.target as HTMLSelectElement;
  setItemValue(props.item, target.value, props.instance);
}
</script>

<template>
  <div class="field suffix no-margin small">
    <select
      :value="getItemValue(item, props.instance) ?? ''"
      :disabled="disabled"
      class="no-round surface-container-highest no-border"
      @change="onChange"
    >
      <option v-for="opt in item.options" :key="opt.value" :value="opt.value">
        {{ opt.label }}
      </option>
    </select>
    <i class="select-icon">keyboard_arrow_down</i>
  </div>
</template>

<style scoped>
.select-icon {
  transform: translate(5px, 0px);
}
</style>
