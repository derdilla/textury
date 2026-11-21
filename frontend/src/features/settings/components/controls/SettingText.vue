<script setup lang="ts">
import { computed } from "vue";
import CopyButton from "@/shared/components/primitives/CopyButton.vue";
import type { SettingItem } from "@/features/settings/utils/settingsSchema";
import type { SettingsInstance } from "@/features/settings/composables/settings";
import { getItemValue } from "@/features/settings/utils/settingsUtils";

const props = defineProps<{
  item: SettingItem;
  disabled?: boolean;
  instance: SettingsInstance<any>;
}>();

const textValue = computed(() => (getItemValue(props.item, props.instance) as string) ?? "");
</script>

<template>
  <div class="row no-margin">
    <div class="field text margin-none small center-align">
      <span
        class="medium-text text-display"
        :class="{ 'mono-text': item.textStyle === 'code' }"
        :title="textValue"
      >
        {{ textValue }}
      </span>
    </div>

    <CopyButton
      v-if="item.textStyle === 'code'"
      :value="textValue"
      :label="item.title"
      :disabled="disabled"
    />
  </div>
</template>

<style scoped>
.text-display {
  max-width: 250px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
