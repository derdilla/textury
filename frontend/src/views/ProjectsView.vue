<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import SidebarLayout from "@/components/general/SidebarLayout.vue";
import CreateProjectModal from "@/components/projects/modals/CreateProjectModal.vue";
import { deleteProject, listProjects, share, whoami, API_ROOT, type ProjectSummary } from "@/services/api/managementApi";
import { editCtrl } from "@/services/editorController";
import { projectCtrl, Sorting, View } from "@/services/projectController.ts";

import ProjectCell from "@/components/projects/ProjectCell.vue";
import ProjectsSearchInput from "@/components/projects/toolbars/ProjectsSearchInput.vue";
import ProjectsTagFilterBar from "@/components/projects/toolbars/ProjectsTagFilterBar.vue";
import ProjectsViewToggle from "@/components/projects/toolbars/ProjectsViewToggle.vue";
import ProjectsSortDropdown from "@/components/projects/toolbars/ProjectsSortDropdown.vue";
import ProjectsAccountDropdown from "@/components/projects/toolbars/ProjectsAccountDropdown.vue";
import ProjectsEmptyStates from "@/components/projects/ProjectsEmptyStates.vue";

const route = useRoute();
const router = useRouter();

const searchQuery = ref("");
const isCreateModalOpen = ref(false);

// Load state from URL query parameters on load.
if (typeof route.query.search === "string") {
  searchQuery.value = route.query.search;
}
if (typeof route.query.tags === "string" && route.query.tags.trim()) {
  projectCtrl.selectedTags = route.query.tags.split(",").filter(Boolean);
} else if (Array.isArray(route.query.tags)) {
  projectCtrl.selectedTags = route.query.tags.filter((t): t is string => typeof t === "string");
}
if (typeof route.query.view === "string" && Object.values(View).includes(route.query.view as View)) {
  projectCtrl.view = route.query.view as View;
}
if (typeof route.query.sort === "string" && Object.values(Sorting).includes(route.query.sort as Sorting)) {
  projectCtrl.sorting = route.query.sort as Sorting;
}

// Sync state changes to URL query parameters.
watch(
  [searchQuery, () => projectCtrl.selectedTags, () => projectCtrl.view, () => projectCtrl.sorting],
  ([newSearch, newTags, newView, newSorting]) => {
    const query: Record<string, string | (string | null)[]> = { ...route.query };

    const trimmedSearch = newSearch.trim();
    if (trimmedSearch) {
      query.search = trimmedSearch;
    } else {
      delete query.search;
    }

    if (newTags.length > 0) {
      query.tags = newTags.join(",");
    } else {
      delete query.tags;
    }

    query.view = newView;
    query.sort = newSorting;

    if (
      query.search !== route.query.search ||
      query.tags !== route.query.tags ||
      query.view !== route.query.view ||
      query.sort !== route.query.sort
    ) {
      router.replace({ query });
    }
  },
  { deep: true, immediate: true }
);

// Sync URL query changes back to local state.
watch(
  () => route.query,
  (newQuery) => {
    const searchFromUrl = typeof newQuery.search === "string" ? newQuery.search : "";
    if (searchQuery.value !== searchFromUrl) {
      searchQuery.value = searchFromUrl;
    }

    let tagsFromUrl: string[] = [];
    if (typeof newQuery.tags === "string" && newQuery.tags.trim()) {
      tagsFromUrl = newQuery.tags.split(",").filter(Boolean);
    } else if (Array.isArray(newQuery.tags)) {
      tagsFromUrl = newQuery.tags.filter((t): t is string => typeof t === "string");
    }

    const tagsChanged =
      tagsFromUrl.length !== projectCtrl.selectedTags.length ||
      tagsFromUrl.some((tag, i) => tag !== projectCtrl.selectedTags[i]);

    if (tagsChanged) {
      projectCtrl.selectedTags = tagsFromUrl;
    }

    if (
      typeof newQuery.view === "string" &&
      Object.values(View).includes(newQuery.view as View) &&
      projectCtrl.view !== newQuery.view
    ) {
      projectCtrl.view = newQuery.view as View;
    }

    if (
      typeof newQuery.sort === "string" &&
      Object.values(Sorting).includes(newQuery.sort as Sorting) &&
      projectCtrl.sorting !== newQuery.sort
    ) {
      projectCtrl.sorting = newQuery.sort as Sorting;
    }
  }
);

// Determines if tags exist in the workspace to keep layout structure consistent regardless of search filter results.
const hasAnyTags = computed(() => {
  return (
    projectCtrl.selectedTags.length > 0 ||
    projectCtrl.projects.some((p) => p.metadata?.tags.length ?? 0 > 0)
  );
});

// Calculate available tags from projects matching selected tags and the search query.
// Incompatible tags (tags not present on any project matching all filters) are hidden.
// Selected tags are always retained so users can deselect them.
// All tags are sorted alphabetically.
const availableTags = computed(() => {
  const query = searchQuery.value.toLowerCase().trim();

  // Find projects that match both all currently selected tags and the search query.
  const matchingProjects = projectCtrl.projects.filter((project) => {
    const currentTags = project.metadata?.tags || [];
    if (!projectCtrl.selectedTags.every((tag) => currentTags.includes(tag))) {
      return false;
    }

    if (query && !project.title?.toLowerCase().includes(query)) {
      return false;
    }

    return true;
  });

  const tagsSet = new Set<string>();

  // Collect tags from projects that match selected tags and search query.
  matchingProjects.forEach((project) => {
    if (Array.isArray(project.metadata?.tags)) {
      project.metadata.tags.forEach((tag) => tagsSet.add(tag));
    }
  });

  // Ensure selected tags are present so users can deselect them.
  projectCtrl.selectedTags.forEach((tag) => tagsSet.add(tag));

  return Array.from(tagsSet).sort((a, b) =>
    a.localeCompare(b, undefined, { sensitivity: "base" })
  );
});

// Filter and sort projects based on selected tags, search query, and sorting criteria.
const filteredProjects = computed(() => {
  let list = projectCtrl.projects.filter((project) => {
    // Multi-Tag Filter
    if (projectCtrl.selectedTags.length > 0) {
      const currentTags = project.metadata?.tags || [];
      if (!projectCtrl.selectedTags.every((tag) => currentTags.includes(tag))) {
        return false;
      }
    }

    // Search Query Filter
    if (searchQuery.value.trim()) {
      const query = searchQuery.value.toLowerCase().trim();
      return project.title?.toLowerCase().includes(query);
    }

    return true;
  });

  // Apply Sorting Logic
  return list.sort((a, b) => {
    switch (projectCtrl.sorting) {
      case Sorting.name:
        return (a.title || "").localeCompare(b.title || "");
      case Sorting.dateEdited:
        return new Date(b.updated_at || 0).getTime() - new Date(a.updated_at || 0).getTime();
      case Sorting.dateCreated:
        return new Date(b.created_at || 0).getTime() - new Date(a.created_at || 0).getTime();
      default:
        return 0;
    }
  });
});

const hasSearch = computed(() => searchQuery.value.trim().length > 0);
const hasTags = computed(() => projectCtrl.selectedTags.length > 0);

function clearSearch() {
  searchQuery.value = "";
}

function clearTags() {
  projectCtrl.selectedTags = [];
}

function clearAllFilters() {
  clearSearch();
  clearTags();
}

async function updateProjectList() {
  const res = await listProjects();
  if (res == null) return;
  projectCtrl.projects = res.projects;
}

function showDialog() {
  isCreateModalOpen.value = true;
}

const shareUrl = ref("");
const shareDialog = ref<HTMLDialogElement>();
async function shareProject(projectId: string) {
  const shareToken = await share(projectId);
  if (!shareToken) return;
  shareUrl.value = `${location.protocol}//${location.hostname}:${location.port}/acceptInvite/${shareToken}`;
  shareDialog.value?.showPopover();
}

async function downloadProject(projectId: string) {
  window.open(`${API_ROOT}/project/${projectId}/zip`, "_self");
}

async function deleteProjectAndUpdate(projectId: string) {
  await deleteProject(projectId);
  updateProjectList();
}

function toggleTag(tag: string) {
  const index = projectCtrl.selectedTags.indexOf(tag);
  if (index > -1) {
    projectCtrl.selectedTags.splice(index, 1);
  } else {
    projectCtrl.selectedTags.push(tag);
  }
}

updateProjectList();

if (!editCtrl.user) {
  whoami().then(async (user) => {
    editCtrl.user = user;
    if (user == null) await router.push("/login");
  });
}
</script>

<template>
  <SidebarLayout hideSidebar contentMaxWidth="1200px" :class="{ 'has-tags': hasAnyTags }">
    <!-- Header Toolbars -->
    <template #content-header>
      <!-- Main Toolbar -->
      <div class="header-container surface row middle-align max">
        <ProjectsAccountDropdown />

        <!-- Share Project Token Dialog -->
        <dialog ref="shareDialog" popover>
          <p>Tell the invited person to go to this url:</p>
          <pre>{{ shareUrl }}</pre>
        </dialog>

        <h6 class="bold view-title">Projects</h6>

        <!-- First Row Tag Bar -->
        <ProjectsTagFilterBar
          v-if="hasAnyTags"
          class="first-row-tag-bar"
          alignment="center"
          :tags="availableTags"
          :selected-tags="projectCtrl.selectedTags"
          @toggle-tag="toggleTag"
        />

        <ProjectsSearchInput v-model="searchQuery" />

        <ProjectsViewToggle
          v-model:view="projectCtrl.view"
          class="first-row-buttons"
        />

        <ProjectsSortDropdown
          v-model:sorting="projectCtrl.sorting"
          class="first-row-buttons"
        />

        <!-- Create Project Button -->
        <button tabindex="0" class="no-round chip medium primary no-border" @click="showDialog">
          <i>add</i>
          <span>Create</span>
        </button>

        <CreateProjectModal v-model:open="isCreateModalOpen" @success="updateProjectList" />
      </div>

      <!-- Second Toolbar (when available space is too small and tags exist) -->
      <div v-if="hasAnyTags" class="header-container top-margin surface row middle-align second-row-container max">
        <ProjectsTagFilterBar
          class="second-row-tag-bar"
          alignment="flex-start"
          :tags="availableTags"
          :selected-tags="projectCtrl.selectedTags"
          @toggle-tag="toggleTag"
        />

        <ProjectsViewToggle
          v-model:view="projectCtrl.view"
          class="second-row-buttons"
        />

        <ProjectsSortDropdown
          v-model:sorting="projectCtrl.sorting"
          class="second-row-buttons"
        />
      </div>
    </template>

    <!-- Projects Grid / List -->
    <div
      class="projects-wrapper"
      :class="{
        'grid-view': projectCtrl.view === View.grid && filteredProjects.length > 0,
        'list-view': projectCtrl.view === View.list || filteredProjects.length === 0,
      }"
    >
      <ProjectCell
        v-for="proj in filteredProjects"
        v-if="filteredProjects.length > 0"
        :key="proj.id"
        :project="proj"
        :view="projectCtrl.view"
        :available-tags="availableTags"
        @delete="deleteProjectAndUpdate"
        @share="shareProject"
        @download="downloadProject"
      />

      <ProjectsEmptyStates
        v-if="filteredProjects.length === 0"
        :has-search="hasSearch"
        :has-tags="hasTags"
        :search-query="searchQuery"
        @clear-search="clearSearch"
        @clear-tags="clearTags"
        @clear-all="clearAllFilters"
        @create-project="showDialog"
      />
    </div>
  </SidebarLayout>
</template>

<style scoped>
.header-container {
  position: relative;
  flex-shrink: 0 !important;
  align-items: center !important;
  box-sizing: border-box;
}

.projects-wrapper {
  display: grid;
  gap: 1rem;
  width: 100%;
}

.projects-wrapper.grid-view {
  grid-template-columns: repeat(auto-fill, minmax(290px, 1fr));
}

.projects-wrapper.list-view {
  grid-template-columns: 1fr;
}

.view-title {
  flex-shrink: 0;
  margin-right: auto;
  min-width: 0;
  overflow: hidden;
}

.second-row-container {
  display: none !important;
}

@media (max-width: 1200px) {
  .has-tags :deep(.first-row-tag-bar),
  .has-tags .first-row-buttons {
    display: none !important;
  }

  .has-tags .second-row-container {
    display: flex !important;
  }
}
</style>