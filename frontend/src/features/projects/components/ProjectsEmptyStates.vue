<script setup lang="ts">
import { computed } from "vue";
import { useProjectsQuery } from "@/features/projects/composables/projectsQuery";
import { useCreateProjectsCondition } from "../composables/createProjectDisabledCondition";

const emit = defineEmits<{
  (e: "create-project"): void;
}>();

const projectsQuery = useProjectsQuery();

const emptyStateMessage = computed(() => {
  const query = projectsQuery.searchQuery.value.trim();
  if (projectsQuery.hasSearchQuery.value && projectsQuery.hasSelectedTagFilters.value) {
    return `No projects found matching "${query}" with the selected tag filters.`;
  }
  if (projectsQuery.hasSearchQuery.value) {
    return `No projects found matching "${query}".`;
  }
  if (projectsQuery.hasSelectedTagFilters.value) {
    return "No projects found matching the selected tag filters.";
  }
  return "You don't have any projects yet. Create your first project to get started!";
});

const { isCreateDisabled, createDisabledReason } = useCreateProjectsCondition();

</script>

<template>
  <article class="medium middle-align center-align">
    <div>
      <i class="extra">{{ (projectsQuery.hasSearchQuery.value || projectsQuery.hasSelectedTagFilters.value) ? 'search' : 'folder_open' }}</i>
      <h5 class="bold">{{ (projectsQuery.hasSearchQuery.value || projectsQuery.hasSelectedTagFilters.value) ? 'No project found' : 'No projects yet' }}</h5>
      <h6 class="small">{{ emptyStateMessage }}</h6>

      <div class="empty-actions">
        <button
          v-if="projectsQuery.hasSearchQuery.value"
          tabindex="0"
          class="chip medium surface-container no-border"
          @click="projectsQuery.clearSearchQuery"
        >
          <i>close</i> Clear search
        </button>

        <button
          v-if="projectsQuery.hasSelectedTagFilters.value"
          tabindex="0"
          class="chip medium surface-container no-border"
          @click="projectsQuery.clearSelectedTagFilters"
        >
          <i>close</i> Clear tag filters
        </button>

        <button
          v-if="projectsQuery.hasSearchQuery.value && projectsQuery.hasSelectedTagFilters.value"
          tabindex="0"
          class="chip medium primary no-border"
          @click="projectsQuery.clearAllFilters"
        >
          <i>close</i> Clear all filters
        </button>

        <button
          v-if="!projectsQuery.hasSearchQuery.value && !projectsQuery.hasSelectedTagFilters.value"
          tabindex="0"
          class="chip medium primary no-border"
          :disabled="isCreateDisabled"
          :title="createDisabledReason || undefined"
          @click="emit('create-project')"
        >
          <i>add</i> Create project
        </button>
      </div>
    </div>
  </article>
</template>

<style scoped>
.empty-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: center;
  margin-top: 1.5rem;
}
</style>
