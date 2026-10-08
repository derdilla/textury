<script setup lang="ts">
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";
import { useProjectsLayoutStore } from "@/features/projects/stores/projectsLayoutStore";

const projectsLayoutStore = useProjectsLayoutStore();
</script>

<template>
  <FloatingElement hover align="center">
    <template #button>
      <button tabindex="0" class="square border" @click="projectsLayoutStore.toggleView">
        <i v-if="projectsLayoutStore.view.value === 'grid'">list</i>
        <i v-if="projectsLayoutStore.view.value === 'list'">grid_view</i>
      </button>
    </template>
    <template #menu>
      <span v-if="projectsLayoutStore.view.value === 'grid'" class="tooltip bottom active">List view</span>
      <span v-if="projectsLayoutStore.view.value === 'list'" class="tooltip bottom active">Grid view</span>
    </template>
  </FloatingElement>
</template>
