<script setup lang="ts">
import FloatingElement from "@/components/general/FloatingElement.vue";
import { View } from "@/services/projectController.ts";

const view = defineModel<View>("view", { required: true });

function toggleView() {
  view.value = view.value === View.grid ? View.list : View.grid;
}
</script>

<template>
  <FloatingElement hover align="center">
    <template #button>
      <button tabindex="0" class="square border" @click="toggleView">
        <i v-if="view === View.grid">list</i>
        <i v-if="view === View.list">grid_view</i>
      </button>
    </template>
    <template #menu>
      <span v-if="view === View.grid" class="tooltip bottom active">List view</span>
      <span v-if="view === View.list" class="tooltip bottom active">Grid view</span>
    </template>
  </FloatingElement>
</template>