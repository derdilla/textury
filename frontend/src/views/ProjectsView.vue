<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { useRoute, useRouter } from "vue-router";
import CreateProjectModal from "@/components/CreateProjectModal.vue";
import UserInviter from "@/components/UserInviter.vue";
import ScrollableContainer from "@/components/ScrollableContainer.vue";
import { deleteProject, listProjects, share, whoami, API_ROOT, type ProjectSummary } from "@/services/managementApi.ts";
import { editCtrl } from "@/services/editorController";
import FloatingElement from "@/components/FloatingElement.vue";
import { invite, logout } from '@/services/managementApi';
import { projectCtrl, Sorting, View } from "@/services/projectController.ts";
import ProjectCell from "@/components/ProjectCell.vue";
import ProjectSearchInput from "@/components/ProjectSearchInput.vue";

const route = useRoute();
const router = useRouter();

const searchQuery = ref("");
const projects = ref<ProjectSummary[]>([]);
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

// Tag Lists
const firstScrollContainer = ref<InstanceType<typeof ScrollableContainer> | null>(null);
const secondScrollContainer = ref<InstanceType<typeof ScrollableContainer> | null>(null);

// Determines if tags exist in the workspace to keep layout structure consistent regardless of search filter results.
const hasAnyTags = computed(() => {
  return (
    projectCtrl.selectedTags.length > 0 ||
    projects.value.some((p) => p.metadata?.tags.length ?? 0 > 0)
  );
});

// Calculate available tags from projects matching selected tags and the search query.
// Incompatible tags (tags not present on any project matching all filters) are hidden.
// Selected tags are always retained so users can deselect them.
// All tags are sorted alphabetically.
const availableTags = computed(() => {
  const query = searchQuery.value.toLowerCase().trim();

  // Find projects that match both all currently selected tags and the search query.
  const matchingProjects = projects.value.filter((project) => {
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

// Trigger recalculation of arrow buttons on tag list.
watch([availableTags, () => projectCtrl.selectedTags], async () => {
  await nextTick();
  firstScrollContainer.value?.triggerCheckScroll();
  secondScrollContainer.value?.triggerCheckScroll();
}, { deep: true });

// Filter and sort projects based on selected tags, search query, and sorting criteria.
const filteredProjects = computed(() => {
  let list = projects.value.filter((project) => {
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

const emptyStateMessage = computed(() => {
  if (hasSearch.value && hasTags.value) {
    return `No projects found matching "${searchQuery.value.trim()}" with the selected tag filters.`;
  }
  if (hasSearch.value) {
    return `No projects found matching "${searchQuery.value.trim()}".`;
  }
  if (hasTags.value) {
    return "No projects found matching the selected tag filters.";
  }
  return "You don't have any projects yet. Create your first project to get started!";
});

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
  projects.value = res.projects;
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
  window.open(`${API_ROOT}/project/${projectId}/zip`, '_self');
}

async function deleteProjectAndUpdate(projectId: string) {
  await deleteProject(projectId);
  updateProjectList();
}

updateProjectList();

if (!editCtrl.user) {
  whoami().then(async (user) => {
    editCtrl.user = user;
    if (user == null) await router.push('/login');
  });
}

const inviteDialog = ref<HTMLDialogElement>();
const registrationToken = ref("");

async function createJoinToken() {
  const success = await invite();
  if (success) {
    registrationToken.value = success.registration_token;
    inviteDialog.value?.showPopover();
  }
}

async function handleLogout() {
  await logout();
  editCtrl.user = null;
  router.push('/login');
}

function toggleTag(tag: string) {
  const index = projectCtrl.selectedTags.indexOf(tag);
  if (index > -1) {
    projectCtrl.selectedTags.splice(index, 1);
  } else {
    projectCtrl.selectedTags.push(tag);
  }
}
</script>

<template>
  <div class="panel surface field" :class="{ 'has-tags': hasAnyTags }">

    <!--Toolbar-->
    <div class="header-container vertical-margin surface row middle-align">

      <!--Account Button + Dropdown-->
      <FloatingElement align="right">
        <template #button>
          <button tabindex="0" class="no-round chip medium secondary-container no-border">
            <i>person</i>
            <span>{{ editCtrl.user?.username }}</span>
          </button>
        </template>
        <template #menu>
          <menu class="dropdown no-wrap no-round active surface-container shadow">
            <button @click="" class="transparent no-round left-align"><i class="small">settings</i> Settings</button>
            <button v-if="editCtrl.user?.is_admin" @click="createJoinToken" class="transparent no-round left-align"><i class="small">person_add</i> Invite user</button>
            <li><hr></li>
            <button @click="handleLogout" class="transparent no-round left-align error-text"><i class="small">logout</i> Log out</button>
          </menu>
        </template>
      </FloatingElement>

      <!--Registration Token Dialog-->
      <dialog popover ref="inviteDialog">
        <p>Tell the invited person to use registration token:</p>
        <pre>{{ registrationToken }}</pre>
      </dialog>

      <!--Share Project Token Dialog-->
      <dialog popover ref="shareDialog">
        <p>Tell the invited person to go to this url:</p>
        <pre>{{ shareUrl }}</pre>
      </dialog>

      <!--View Title-->
      <h6 class="bold view-title">Projects</h6>

      <!--Project Tag Buttons-->
      <ScrollableContainer v-if="hasAnyTags" ref="firstScrollContainer" content-alignment="center" class="first-row-tag-bar">
        <button @click="toggleTag(tag)" v-for="tag in availableTags" :key="tag" class="chip medium no-border no-round tag"
          :class="{ 'primary': projectCtrl.selectedTags.includes(tag), 'surface-container': !projectCtrl.selectedTags.includes(tag) }">
          {{ tag }}
        </button>
      </ScrollableContainer>

      <!--Search Projects Box-->
      <ProjectSearchInput v-model="searchQuery" />

      <!--Change View Button + Tooltip-->
      <FloatingElement hover align="center" class="first-row-buttons">
        <template #button>
          <button tabindex="0" @click="projectCtrl.view = projectCtrl.view === View.grid ? View.list : View.grid" class="square border">
            <i v-if="projectCtrl.view === View.grid">list</i>
            <i v-if="projectCtrl.view === View.list">grid_view</i>
          </button>
        </template>
        <template #menu>
          <span v-if="projectCtrl.view === View.grid" class="tooltip bottom active">List view</span>
          <span v-if="projectCtrl.view === View.list" class="tooltip bottom active">Grid view</span>
        </template>
      </FloatingElement>

      <!--Sort Projects Button + Tooltip + Dropdown-->
      <FloatingElement hover align="center" ref="first-row-sort-dropdown" class="first-row-buttons">
        <template #button>
          <FloatingElement align="left">
            <template #button>
              <button tabindex="0" @click="$refs['first-row-sort-dropdown'].isOpen = false" class="square border">
                <i>sort</i>
              </button>
            </template>
            <template #menu>
              <menu class="dropdown no-wrap no-round active surface-container shadow">
                <span class="dropdown-title grey-text left-padding" style="padding-top: 8px; padding-bottom: 5px;">Sort by</span>
                <li><hr></li>
                <button @click="projectCtrl.sorting = Sorting.name" 
                  :class="{ 'secondary-container': projectCtrl.sorting === Sorting.name }"
                  class="transparent no-round left-align">
                  <i class="small">sort_by_alpha</i> Name
                </button>
                <button @click="projectCtrl.sorting = Sorting.dateEdited" 
                  :class="{ 'secondary-container': projectCtrl.sorting === Sorting.dateEdited }"
                  class="transparent no-round left-align">
                  <i class="small">update</i> Date edited
                </button>
                <button @click="projectCtrl.sorting = Sorting.dateCreated" 
                  :class="{ 'secondary-container': projectCtrl.sorting === Sorting.dateCreated }"
                  class="transparent no-round left-align">
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
      
      <!--Create Project Button-->
      <button tabindex="0" @click="showDialog" class="no-round chip medium primary no-border">
        <i>add</i>
        <span>Create</span>
      </button>

      <!--Create Project Modal-->
      <CreateProjectModal v-model="isCreateModalOpen" @success="updateProjectList" />

    </div>

    <!--Second Toolbar (when available space is too small and tags exist)-->
    <div v-if="hasAnyTags" class="header-container bottom-margin surface row middle-align second-row-container">
      <ScrollableContainer ref="secondScrollContainer" content-alignment="flex-start" class="second-row-tag-bar">
        <button @click="toggleTag(tag)" v-for="tag in availableTags" :key="tag" class="chip medium no-border no-round tag"
          :class="{ 'primary': projectCtrl.selectedTags.includes(tag), 'surface-container': !projectCtrl.selectedTags.includes(tag) }">
          {{ tag }}
        </button>
      </ScrollableContainer>

      <FloatingElement hover align="center" class="second-row-buttons">
        <template #button>
          <button tabindex="0" @click="projectCtrl.view = projectCtrl.view === View.grid ? View.list : View.grid" class="square border">
            <i v-if="projectCtrl.view === View.grid">list</i>
            <i v-if="projectCtrl.view === View.list">grid_view</i>
          </button>
        </template>
        <template #menu>
          <span v-if="projectCtrl.view === View.grid" class="tooltip bottom active">List view</span>
          <span v-if="projectCtrl.view === View.list" class="tooltip bottom active">Grid view</span>
        </template>
      </FloatingElement>

      <FloatingElement hover align="center" ref="second-row-sort-dropdown" class="second-row-buttons">
        <template #button>
          <FloatingElement align="left">
            <template #button>
              <button tabindex="0" @click="$refs['second-row-sort-dropdown'].isOpen = false" class="square border">
                <i>sort</i>
              </button>
            </template>
            <template #menu>
              <menu class="dropdown no-wrap no-round active surface-container shadow">
                <span class="dropdown-title grey-text left-padding" style="padding-top: 8px; padding-bottom: 5px;">Sort by</span>
                <li><hr></li>
                <button @click="projectCtrl.sorting = Sorting.name" 
                  :class="{ 'secondary-container': projectCtrl.sorting === Sorting.name }"
                  class="transparent no-round left-align">
                  <i class="small">sort_by_alpha</i> Name
                </button>
                <button @click="projectCtrl.sorting = Sorting.dateEdited" 
                  :class="{ 'secondary-container': projectCtrl.sorting === Sorting.dateEdited }"
                  class="transparent no-round left-align">
                  <i class="small">update</i> Date edited
                </button>
                <button @click="projectCtrl.sorting = Sorting.dateCreated" 
                  :class="{ 'secondary-container': projectCtrl.sorting === Sorting.dateCreated }"
                  class="transparent no-round left-align">
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
    </div>

    <!--Projects-->
    <div class="main-container max small-scroll-bar">
      <div :class="{ 'grid-view': projectCtrl.view === View.grid && filteredProjects.length > 0, 'list-view': projectCtrl.view === View.list || filteredProjects.length == 0 }" class="projects-wrapper">
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
        <article v-if="filteredProjects.length == 0" class="medium middle-align center-align">
          <div>
            <i class="extra">{{ (hasSearch || hasTags) ? 'search' : 'folder_open' }}</i>
            <h5 class="bold">{{ (hasSearch || hasTags) ? 'No project found' : 'No projects yet' }}</h5>
            <h6 class="small">{{ emptyStateMessage }}</h6>

            <div class="empty-actions">
              <button tabindex="0" v-if="hasSearch" @click="clearSearch" class="chip medium surface-container no-border">
                <i>close</i> Clear search
              </button>
              <button tabindex="0" v-if="hasTags" @click="clearTags" class="chip medium surface-container no-border">
                <i>close</i> Clear tag filters
              </button>
              <button tabindex="0" v-if="hasSearch && hasTags" @click="clearAllFilters" class="chip medium primary no-border">
                <i>close</i> Clear all filters
              </button>
              <button tabindex="0" v-if="!hasSearch && !hasTags" @click="showDialog" class="chip medium primary no-border">
                <i>add</i> Create project
              </button>
            </div>
          </div>
        </article>
      </div>
    </div>

  </div>
</template>

<style scoped>
.panel {
  height: 100vh;
  color-scheme: dark;
}

.header-container {
  position: relative;
  flex-grow: 0 !important;
  align-items: center !important;
}

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

.main-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  overflow: auto;
  width: 100%;
}

.small-scroll-bar {
  scrollbar-gutter: stable;
  scrollbar-width: thin;
}

.projects-wrapper {
  display: grid;
  gap: 1rem;
  padding: 1.5rem;
  max-width: 1200px;
  margin: 0 auto;
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

.empty-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: center;
  margin-top: 1.5rem;
}

@media (max-width: 1200px) {
  .has-tags .first-row-tag-bar,
  .has-tags .first-row-buttons {
    display: none !important;
  }

  .has-tags .second-row-container {
    display: flex !important;
  }
}
</style>