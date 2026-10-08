<script setup lang="ts">
import { ref, computed } from "vue";
import type { SettingItem } from "@/features/settings/utils/settingsSchema";
import type { SettingsInstance } from "@/features/settings/composables/settings";
import { isDisabled, getDisabledReason, getItemValue, setItemValue } from "@/features/settings/utils/settingsUtils";
import DisabledHelp from "@/features/settings/components/DisabledHelp.vue";

import SettingSelect from "@/features/settings/components/controls/SettingSelect.vue";
import SettingSwitch from "@/features/settings/components/controls/SettingSwitch.vue";
import SettingSlider from "@/features/settings/components/controls/SettingSlider.vue";
import SettingRange from "@/features/settings/components/controls/SettingRange.vue";
import SettingTextInput from "@/features/settings/components/controls/SettingTextInput.vue";
import SettingAction from "@/features/settings/components/controls/SettingAction.vue";

const props = defineProps<{
  item: SettingItem;
  isFirstInGroup?: boolean;
  hasOpenAccordionAbove?: boolean;
  instance: SettingsInstance<any>;
}>();

const emit = defineEmits<{
  (e: "openPage", item: SettingItem): void;
}>();

const actionRef = ref<InstanceType<typeof SettingAction> | null>(null);

const disabled = computed(() => isDisabled(props.item.disabled, props.instance));
const disabledReasonText = computed(() => getDisabledReason(props.item.disabledReason, props.instance));

const isClickableRow = computed(() => {
  if (props.item.type === "action" && props.item.actionStyle === "button") {
    return false;
  }
  return ["switch", "page", "action"].includes(props.item.type);
});

function handleRowClick() {
  if (disabled.value) return;

  if (props.item.type === "switch") {
    setItemValue(props.item, !getItemValue(props.item, props.instance), props.instance);
  } else if (props.item.type === "page") {
    emit("openPage", props.item);
  } else if (props.item.type === "action" && props.item.actionStyle !== "button") {
    actionRef.value?.executeAction();
  }
}

const resolvedCustomProps = computed(() => {
  if (typeof props.item.customProps === "function") {
    return props.item.customProps(props.instance);
  }
  return props.item.customProps ?? {};
});
</script>

<template>
  <!-- Custom Row -->
  <component
    :is="item.customComponent"
    v-if="item.type === 'custom' && item.customComponent"
    :instance="instance"
    v-bind="resolvedCustomProps"
  />

  <!-- Predefined Row -->
  <div v-else
    class="row padding middle-align content-row"
    :class="[
      { 'has-open-accordion-above': hasOpenAccordionAbove },
      { 'disabled': disabled && !isClickableRow },
      { 'error-text': item.type === 'action' && item.destructive },
      { 'wave pointer': isClickableRow && !disabled }
    ]"
    @click="handleRowClick"
  >
    <!-- Icon & Titles -->
    <i
      v-if="item.icon || item.reserveIconSpace"
      class="medium"
      :class="{ 'icon-hidden': !item.icon }"
    >
      {{ item.icon }}
    </i>
    <div class="max">
      <div :class="{ bold: item.type === 'action' && item.destructive }">
        {{ item.title }}
      </div>
      <div v-if="item.description" class="small-text text-secondary">
        {{ item.description }}
      </div>
    </div>

    <!-- Disabled State Tooltip -->
    <DisabledHelp v-if="disabled && disabledReasonText.length > 0" :reason="disabledReasonText" />

    <!-- Control Elements -->
    <span v-if="item.type === 'text'" class="medium-text">
      {{ getItemValue(item, props.instance) }}
    </span>

    <SettingTextInput
      v-else-if="item.type === 'textinput'"
      :item="item"
      :disabled="disabled"
      :instance="instance"
    />

    <SettingSelect
      v-else-if="item.type === 'select'"
      :item="item"
      :disabled="disabled"
      :instance="instance"
    />

    <SettingSwitch
      v-else-if="item.type === 'switch'"
      :item="item"
      :disabled="disabled"
      :instance="instance"
    />

    <SettingSlider
      v-else-if="item.type === 'slider'"
      :item="item"
      :disabled="disabled"
      :is-first-in-group="isFirstInGroup"
      :instance="instance"
    />

    <SettingRange
      v-else-if="item.type === 'range'"
      :item="item"
      :disabled="disabled"
      :is-first-in-group="isFirstInGroup"
      :instance="instance"
    />

    <SettingAction
      v-else-if="item.type === 'action'"
      ref="actionRef"
      :item="item"
      :disabled="disabled"
      :instance="instance"
    />

    <i v-else-if="item.type === 'page'" :class="{ 'disabled-icon': disabled }">
      arrow_forward
    </i>
  </div>
</template>

<style scoped>
.content-row {
  border-radius: 0;
  margin: 0;
  width: 100%;
  box-sizing: border-box;
  transition: padding-top 0.2s ease;
}

.disabled-icon {
  opacity: 0.38;
  cursor: not-allowed;
}

.icon-hidden {
  visibility: hidden;
}

.has-open-accordion-above {
  padding-top: calc(1rem + 18px) !important;
}

.pointer {
  cursor: pointer;
}

.text-secondary {
  opacity: 0.7;
  font-size: 0.85rem;
}
</style>
