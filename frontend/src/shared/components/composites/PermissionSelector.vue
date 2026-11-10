<script setup lang="ts">
import type { ProjectPermissionLevel } from "@/api/types";

const modelValue = defineModel<ProjectPermissionLevel>({ required: true });

const permissionOptions: Record<ProjectPermissionLevel, { title: string; description: string }> = {
  edit: { title: "Editor", description: "Can edit, compile and view project files." },
  compile: { title: "Compile", description: "Can compile and view project files." },
  view: { title: "Viewer", description: "Can only view project files." },
};
</script>

<template>
  <div class="grid no-margin permission-selector-grid">
    <button
      v-for="(option, key) in permissionOptions"
      :key="key"
      type="button"
      class="no-round no-margin small-padding max permission-card left-align s12"
      :class="modelValue === key ? 'primary' : 'surface-container-highest'"
      @click="modelValue = key"
    >
      <div class="row middle-align no-margin max">
        <i class="radio-icon">
          {{ modelValue === key ? "radio_button_checked" : "radio_button_unchecked" }}
        </i>

        <div class="max left-align">
          <div class="bold">{{ option.title }}</div>
          <div class="small-text text-secondary">{{ option.description }}</div>
        </div>
      </div>
    </button>
  </div>
</template>

<style scoped>
.permission-selector-grid {
  gap: 0.75rem;
}

.permission-card {
  transition: background-color 0.25s ease, color 0.25s ease;
}

.radio-icon {
  margin-left: 0.4rem;
  margin-right: -0.1rem;
}

.text-secondary {
  opacity: 0.7;
}
</style>
