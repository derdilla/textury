<script setup lang="ts">
import { ref, computed } from "vue";
import SidebarLayout from "@/shared/components/layouts/SidebarLayout.vue";
import CreateProjectModal from "@/features/projects/components/modals/CreateProjectModal.vue";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import { useAuthStore } from "@/shared/stores/authStore";
import { useProjectsQuery } from "@/features/projects/composables/projectsQuery";

import ProjectCell from "@/features/projects/components/ProjectCell.vue";
import ProjectsSearchInput from "@/features/projects/components/toolbars/ProjectsSearchInput.vue";
import ProjectsTagFilterBar from "@/features/projects/components/toolbars/ProjectsTagFilterBar.vue";
import ProjectsViewToggle from "@/features/projects/components/toolbars/ProjectsViewToggle.vue";
import ProjectsSortDropdown from "@/features/projects/components/toolbars/ProjectsSortDropdown.vue";
import ProjectsAccountDropdown from "@/features/projects/components/toolbars/ProjectsAccountDropdown.vue";
import ProjectsEmptyStates from "@/features/projects/components/ProjectsEmptyStates.vue";

import logo_dark from '@/shared/assets/logo_dark.svg';
import logo_light from '@/shared/assets/logo_light.svg';
import { useProjectsLayoutStore } from "@/features/projects/stores/projectsLayoutStore";

const projectsStore = useProjectsStore();
const projectsQuery = useProjectsQuery();
const projectsLayout = useProjectsLayoutStore();
const authStore = useAuthStore();

const isCreateDisabled = computed<boolean>(() => createDisabledReason.value !== null);
const createDisabledReason = computed<string | null>(() => {
  if (!authStore.state.user) return "No user loaded";
  if (authStore.state.user.tiedToProject) return "You are tied to a project";
  return null;
});

const isCreateModalOpen = ref(false);

function showDialog() {
  isCreateModalOpen.value = true;
}

if (!projectsStore.state.isInitialized) {
  projectsStore.fetchProjects();
}
</script>

<template>
  <SidebarLayout hideSidebar contentMaxWidth="1200px" :class="{ 'has-tags': projectsQuery.hasAnyTags.value }">
    <!-- Header Toolbars -->
    <template #content-header>
      <!-- Main Toolbar -->
      <div class="header-container surface row middle-align max">
        <ProjectsAccountDropdown />

        <img :src="logo_dark" alt="Textury Logo" width="100px" class="view-title dark-only"/>
        <img :src="logo_light" alt="Textury Logo" width="100px" class="view-title light-only"/>

        <!-- First Row Tag Bar -->
        <ProjectsTagFilterBar
          v-if="projectsQuery.hasAnyTags.value"
          class="first-row-tag-bar"
          alignment="center"
        />

        <ProjectsSearchInput/>

        <ProjectsViewToggle class="first-row-buttons"/>

        <ProjectsSortDropdown class="first-row-buttons"/>

        <!-- Create Project Button -->
        <button tabindex="0" class="no-round chip medium primary no-border"
          @click="showDialog"
          :title="createDisabledReason || undefined"
          :disabled="isCreateDisabled">
          <i>add</i>
          <span>Create</span>
        </button>

        <CreateProjectModal v-model:open="isCreateModalOpen"/>
      </div>

      <!-- Second Toolbar (when available space is too small and tags exist) -->
      <div v-if="projectsQuery.hasAnyTags.value" class="header-container top-margin surface row middle-align second-row-container max">
        <ProjectsTagFilterBar
          class="second-row-tag-bar"
          alignment="flex-start"
        />

        <ProjectsViewToggle class="second-row-buttons"/>

        <ProjectsSortDropdown class="second-row-buttons"/>
      </div>
    </template>

    <!-- Projects Grid / List -->
    <div
      class="projects-wrapper"
      :class="{
        'grid-view': projectsLayout.view.value === 'grid' && projectsQuery.filteredProjects.value.length > 0,
        'list-view': projectsLayout.view.value === 'list' || projectsQuery.filteredProjects.value.length === 0,
      }"
    >
      <ProjectCell
        v-for="proj in projectsQuery.filteredProjects.value"
        :key="proj.id"
        :project-id="proj.id"
      />

      <ProjectsEmptyStates
        v-if="projectsQuery.filteredProjects.value.length === 0"
        @create-project="showDialog"
      />
    </div>
  </SidebarLayout>
</template>

<style scoped>
.dark-only { display: none; }
.light-only { display: block; }

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

.dark .dark-only { display: block; }
.dark .light-only { display: none; }
</style>
