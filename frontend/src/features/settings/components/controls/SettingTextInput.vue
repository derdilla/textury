<script setup lang="ts">
import { ref, computed } from "vue";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";
import type { SettingItem } from "@/features/settings/utils/settingsSchema";
import type { SettingsInstance } from "@/features/settings/composables/settings";
import { getItemValue, setItemValue } from "@/features/settings/utils/settingsUtils";

const props = defineProps<{
  item: SettingItem;
  disabled?: boolean;
  instance: SettingsInstance<any>;
}>();

const isEditing = ref(false);

const textValue = computed({
  get: () => (getItemValue(props.item, props.instance) as string) ?? "",
  set: (val: string) => {
    setItemValue(props.item, val, props.instance);
  },
});

function toggleEdit() {
  if (props.disabled) return;
  isEditing.value = !isEditing.value;
}

function handleEnter(event: Event) {
  (event.target as HTMLInputElement).blur();
  isEditing.value = false;
}
</script>

<template>
  <div class="row no-margin">
    <div class="field text margin-none small center-align">
      <input v-if="isEditing"
        v-model="textValue"
        :type="item.inputType ?? 'text'"
        :placeholder="item.placeholder ?? ''"
        :disabled="disabled"
        @keydown.enter="handleEnter"
        @click.stop
        class="no-round surface-container-highest no-border text-input"
      />
      <span v-else class="large-text text-display">
        {{ textValue || item.placeholder || '' }}
      </span>
    </div>

    <FloatingElement hover align="left">
      <template #button>
        <button class="square" :disabled="disabled" @click="toggleEdit" :class="{ 'primary': isEditing, 'border': !isEditing }">
          <i>{{ isEditing ? 'check' : 'edit' }}</i>
        </button>
      </template>
      <template #menu>
        <span class="tooltip bottom active">{{ isEditing ? `Save ${item.title}` : `Edit ${item.title}` }}</span>
      </template>
    </FloatingElement>
  </div>
</template>

<style scoped>
.text-display {
  max-width: 250px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.text-input {
  min-width: 250px;
}
</style>
