<script setup lang="ts">
import { computed } from "vue";
import type { SettingItem } from "@/features/settings/utils/settingsSchema";
import type { SettingsInstance } from "@/features/settings/composables/settings";
import { isDisabled, getDisabledReason, getVisibleItems } from "@/features/settings/utils/settingsUtils";
import DisabledHelp from "@/features/settings/components/DisabledHelp.vue";
import SettingRow from "@/features/settings/components/SettingRow.vue";

const props = defineProps<{
  item: SettingItem;
  hasOpenAccordionAbove?: boolean;
  hasItemBelow?: boolean;
  instance: SettingsInstance<any>;
}>();

const emit = defineEmits<{
  (e: "openPage", item: SettingItem): void;
}>();

const openModel = defineModel<boolean>("open", { default: false });

const disabled = computed(() => isDisabled(props.item.disabled, props.instance));
const disabledReasonText = computed(() => getDisabledReason(props.item.disabledReason, props.instance));

const isOpen = computed(() => openModel.value && !disabled.value);

const visibleChildItems = computed(() => getVisibleItems(props.item.items, props.instance));

function toggle() {
  if (disabled.value) return;
  openModel.value = !openModel.value;
}
</script>

<template>
  <div class="accordion-wrapper" :class="{ 'is-open': isOpen }">
    <div class="accordion-item">
      <div
        class="accordion-header content-row row padding middle-align"
        :class="[
          { 'has-open-accordion-above': hasOpenAccordionAbove },
          { 'wave pointer': !disabled }
        ]"
        @click="toggle"
      >
        <i class="medium">{{ item.icon }}</i>
        <div class="max">
          <div>{{ item.title }}</div>
          <div v-if="item.description" class="small-text text-secondary">
            {{ item.description }}
          </div>
        </div>

        <DisabledHelp v-if="disabled" :reason="disabledReasonText" />

        <i
          class="expand-icon"
          :class="[
            { 'expanded': isOpen },
            { 'disabled-icon': disabled }
          ]"
        >
          expand_more
        </i>
      </div>

      <div
        v-if="isOpen"
        class="accordion-body"
        :class="{ 'has-item-below': hasItemBelow }"
      >
        <template v-for="(child, cIndex) in visibleChildItems" :key="child.id">
          <hr v-if="cIndex > 0" />
          <SettingRow
            :item="child"
            :is-first-in-group="cIndex === 0"
            :instance="instance"
            @open-page="(subPage) => emit('openPage', subPage)"
          />
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.accordion-wrapper {
  width: 100%;
}

.accordion-item {
  display: flex;
  flex-direction: column;
  width: 100%;
  position: relative;
}

.accordion-header {
  position: relative;
  z-index: 1;
  transition: padding-bottom 0.2s ease, padding-top 0.2s ease;
}

.accordion-header.has-open-accordion-above {
  padding-top: calc(1rem + 18px) !important;
}

.accordion-wrapper.is-open .accordion-header {
  padding-bottom: calc(1rem + 18px) !important;
}

.content-row {
  border-radius: 0;
  margin: 0;
  width: 100%;
  box-sizing: border-box;
}

.pointer {
  cursor: pointer;
}

.text-secondary {
  opacity: 0.7;
  font-size: 0.85rem;
}

.disabled-icon {
  opacity: 0.38;
  cursor: not-allowed;
}

.accordion-body {
  position: relative;
  z-index: 10;
  margin-top: -18px;
  margin-bottom: 0;
  background-color: var(--surface-container-high);
  border-top: 1px solid var(--outline-variant);
  border-bottom: none;
  border-radius: 18px;
  overflow: hidden;
}

.accordion-body.has-item-below {
  margin-bottom: -18px;
  border-bottom: 1px solid var(--outline-variant);
}

.expand-icon {
  transition: transform 0.25s ease;
}

.expand-icon.expanded {
  transform: rotate(180deg);
}
</style>
