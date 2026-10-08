<script setup lang="ts">
import { ref } from "vue";
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";
import { useProjectsQuery } from "@/features/projects/composables/projectsQuery";

const projectsQuery = useProjectsQuery();

const outerFloating = ref();

function handleButtonClick() {
  if (outerFloating.value) {
    outerFloating.value.isOpen = false;
  }
}

</script>

<template>
  <FloatingElement ref="outerFloating" hover align="center">
    <template #button>
      <FloatingElement align="left">
        <template #button>
          <button tabindex="0" class="square border" @click="handleButtonClick">
            <i>sort</i>
          </button>
        </template>
        <template #menu>
          <menu class="dropdown no-wrap no-round active surface-container">
            <span class="dropdown-title grey-text left-padding" style="padding-top: 8px; padding-bottom: 5px;">
              Sort by
            </span>
            <li><hr /></li>
            <button
              class="transparent no-round left-align"
              :class="{ 'secondary-container': projectsQuery.sortBy.value === 'title' }"
              @click="projectsQuery.setSortBy('title')"
            >
              <i class="small">sort_by_alpha</i> Name
            </button>
            <button
              class="transparent no-round left-align"
              :class="{ 'secondary-container': projectsQuery.sortBy.value === 'updatedAt' }"
              @click="projectsQuery.setSortBy('updatedAt')"
            >
              <i class="small">update</i> Date edited
            </button>
            <button
              class="transparent no-round left-align"
              :class="{ 'secondary-container': projectsQuery.sortBy.value === 'createdAt' }"
              @click="projectsQuery.setSortBy('createdAt')"
            >
              <i class="small">calendar_today</i> Date created
            </button>
          </menu>
        </template>
      </FloatingElement>
    </template>
    <template #menu>
      <span class="tooltip bottom active">Sorting</span>
    </template>
  </FloatingElement>
</template>

<style scoped>
.dropdown hr {
  margin: 3px;
}
.dropdown i {
  margin-right: 0.5rem;
}
.dropdown {
  gap: 4px;
}
.dropdown-title {
  margin-bottom: -2px;
}
.dropdown {
  box-shadow: none !important;
}
</style>
