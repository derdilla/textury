<script setup lang="ts">
import { computed } from "vue";
import type { SettingItem } from "@/components/settings/settingsSchema";
import type { SettingsInstance } from "@/composables/settings";
import { getItemValue, setItemValue } from "@/components/settings/settingsUtils";

const props = defineProps<{
  item: SettingItem;
  disabled?: boolean;
  instance: SettingsInstance<any>;
}>();

const textValue = computed({
  get: () => (getItemValue(props.item, props.instance) as string) ?? "",
  set: (val: string) => {
    setItemValue(props.item, val, props.instance);
  },
});

function handleEnter(event: Event) {
  (event.target as HTMLInputElement).blur();
}
</script>

<template>
  <div class="field text margin-none small">
    <input
      v-model="textValue"
      :type="item.inputType ?? 'text'"
      :placeholder="item.placeholder ?? ''"
      :disabled="disabled"
      @keydown.enter="handleEnter"
      @click.stop
      class="no-round surface-container-highest no-border"
    />
  </div>
</template>

<style scoped>
.small-input {
  min-width: 180px;
  max-width: 260px;
}
</style>