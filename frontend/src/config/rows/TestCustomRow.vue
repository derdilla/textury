<script setup lang="ts">
import type { SettingsStore } from "@/config/settingsEngine";
import type { SettingItem } from "@/components/settings/settingsSchema";

const props = defineProps<{
  item?: SettingItem;
  store: SettingsStore<any>;
  badgeText?: string;
}>();

function toggle() {
  props.store.rawValues.test_custom_with_props = !props.store.rawValues.test_custom_with_props;
}
</script>

<template>
  <div class="row middle-align padding surface-container margin-bottom">
    <i v-if="item?.icon" class="margin-right">{{ item.icon }}</i>
    <span v-if="badgeText" class="chip margin-right no-round">{{ badgeText }}</span>

    <div class="max">
      <div class="bold">{{ item?.title ?? "Custom Row" }}</div>
      <div v-if="item?.description" class="text-small opacity-70">
        {{ item.description }}
      </div>
    </div>

    <button class="border no-round" @click="toggle">
      Toggle ({{ store.rawValues.test_custom_with_props ? "ON" : "OFF" }})
    </button>
  </div>
</template>