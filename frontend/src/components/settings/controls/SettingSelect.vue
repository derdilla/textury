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
  const target = e.target as HTMLSelectElement;
  setItemValue(props.item, target.value, props.store);
}
</script>

<template>
  <div class="field suffix no-margin small">
    <select
      :value="getItemValue(item, props.store) ?? ''"
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