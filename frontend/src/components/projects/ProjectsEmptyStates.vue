<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  hasSearch: boolean;
  hasTags: boolean;
  searchQuery: string;
}>();

const emit = defineEmits<{
  (e: "clear-search"): void;
  (e: "clear-tags"): void;
  (e: "clear-all"): void;
  (e: "create-project"): void;
}>();

const emptyStateMessage = computed(() => {
  if (props.hasSearch && props.hasTags) {
    return `No projects found matching "${props.searchQuery.trim()}" with the selected tag filters.`;
  }
  if (props.hasSearch) {
    return `No projects found matching "${props.searchQuery.trim()}".`;
  }
  if (props.hasTags) {
    return "No projects found matching the selected tag filters.";
  }
  return "You don't have any projects yet. Create your first project to get started!";
});
</script>

<template>
  <article class="medium middle-align center-align">
    <div>
      <i class="extra">{{ (hasSearch || hasTags) ? 'search' : 'folder_open' }}</i>
      <h5 class="bold">{{ (hasSearch || hasTags) ? 'No project found' : 'No projects yet' }}</h5>
      <h6 class="small">{{ emptyStateMessage }}</h6>

      <div class="empty-actions">
        <button
          v-if="hasSearch"
          tabindex="0"
          class="chip medium surface-container no-border"
          @click="emit('clear-search')"
        >
          <i>close</i> Clear search
        </button>
        <button
          v-if="hasTags"
          tabindex="0"
          class="chip medium surface-container no-border"
          @click="emit('clear-tags')"
        >
          <i>close</i> Clear tag filters
        </button>
        <button
          v-if="hasSearch && hasTags"
          tabindex="0"
          class="chip medium primary no-border"
          @click="emit('clear-all')"
        >
          <i>close</i> Clear all filters
        </button>
        <button
          v-if="!hasSearch && !hasTags"
          tabindex="0"
          class="chip medium primary no-border"
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