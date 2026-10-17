<script setup lang="ts">
import { ref, computed } from 'vue';
import { useProjectsStore } from '@/shared/stores/projectsStore';
import { useContactsStore } from '@/shared/stores/contactsStore';
import { useAuthStore } from '@/shared/stores/authStore';
import { formatTimeAgo } from '@/shared/utils/formatters';
import FloatingElement from "@/shared/components/primitives/FloatingElement.vue";
import TagList from '@/shared/components/composites/TagList.vue';
import ProjectTagsModal from "@/features/projects/components/modals/ProjectTagsModal.vue";
import ProjectSettingsModal from "@/features/settings/views/ProjectSettingsModal.vue";
import ConfirmationModal from "@/shared/components/composites/ConfirmationModal.vue";
import { useProjectsLayoutStore } from '@/features/projects/stores/projectsLayoutStore';
import { API_ROOT } from "@/api";

const props = defineProps<{
  projectId: string;
}>();

const projectsStore = useProjectsStore();
const projectsLayoutStore = useProjectsLayoutStore();
const contactsStore = useContactsStore();
const authStore = useAuthStore();

const project = computed(() => projectsStore.getProjectById(props.projectId));

const isMenuOpen = ref(false);
const isTagsModalOpen = ref(false);
const isProjectSettingsModalOpen = ref(false);
const isDeleteProjectModalOpen = ref(false);
const floatingElement = ref();

const shareUrl = ref('');
const shareDialog = ref<HTMLDialogElement>();

const closeMenu = () => { if (floatingElement.value) floatingElement.value.isOpen = false; };

const handleDelete = () => (closeMenu(), projectsStore.deleteProject(props.projectId));
const handleDownload = () => (closeMenu(), window.open(`${API_ROOT}/project/${props.projectId}/zip`, '_self'));

async function handleShare() {
  closeMenu();
  const res = await projectsStore.createShareToken(props.projectId);
  if (res.success && res.data) {
    shareUrl.value = `${window.location.origin}/acceptInvite/${res.data.token}`;
    shareDialog.value?.showPopover?.();
  }
}

const projectTags = computed(() => project.value?.metadata?.tags ?? []);
const isOwner = computed(() => project.value?.owner === authStore.state.user?.id);
const updatedByName = computed(() => project.value ? contactsStore.getUsername(project.value.updatedBy) : '');

const memberNames = computed(() => {
  if (!project.value) return [];
  const members = [project.value.owner, ...(project.value.members || []).filter(id => id !== project.value?.owner)];
  return members.map(id => contactsStore.getUsername(id));
});
</script>

<template>
  <div v-if="project" :class="['surface-container project-item padding', projectsLayoutStore.view.value === 'grid' ? 'grid-cell' : 'list-cell', isMenuOpen ? 'menu-active' : '']">

    <RouterLink :to="`/projects/${project.id}`" class="project-link max relative no-gap" :class="{ 'row': projectsLayoutStore.view.value === 'list', 'field top-align left-align': projectsLayoutStore.view.value === 'grid' }">

      <!-- Project Icon -->
      <div v-if="projectsLayoutStore.view.value === 'list'" class="shape secondary-container no-round small">
        <i class="medium">description</i>
      </div>
      <div v-if="projectsLayoutStore.view.value === 'grid'" class="secondary-container no-round row center-align grid-image">
        <i class="extra">description</i>
      </div>

      <!-- Project Title + Members -->
      <div class="project-text field">
        <h5 class="small truncate-multi-line bold">{{ project.title }}</h5>
        <TagList :tags="memberNames" :max-lines="1" :max-tags="1">
          <template #tag="{ tag }">
            <span class="grey-border" style="opacity: 0.8;">{{ tag }}</span>
          </template>
          <template #badge="{ count }">
            <span class="italic grey-text">+{{ count }}</span>
          </template>
        </TagList>
      </div>

      <!-- Project Tags + Date -->
      <div class="project-info field">
        <TagList :tags="projectTags" :max-lines="1" :max-tags="3" v-if="projectTags.length">
          <template #tag="{ tag }">
            <div class="secondary-container tag">{{ tag }}</div>
          </template>
          <template #badge="{ count }">
            <div class="surface-container-highest tag italic">+{{ count }}</div>
          </template>
        </TagList>
        <span class="surface-container-highest edited-stamp">
          {{ formatTimeAgo(project.updatedAt) }} by {{ updatedByName }}
        </span>
      </div>

    </RouterLink>

    <!-- Dropdown -->
    <FloatingElement align="left" class="absolute right right-margin" :class="{ 'middle': projectsLayoutStore.view.value === 'list', 'bottom bottom-margin': projectsLayoutStore.view.value === 'grid' }" @click.stop @toggle="isMenuOpen = $event" ref="floatingElement">
      <template #button>
        <button tabindex="0" class="square transparent more-button">
          <i>more_vert</i>
        </button>
      </template>
      <template #menu>
        <menu class="dropdown no-wrap no-round active surface-container">
          <button @click.stop.prevent="closeMenu(); isTagsModalOpen = true" class="transparent no-round left-align">
            <i class="small">{{ projectTags.length ? 'label' : 'new_label' }}</i> {{ projectTags.length ? 'Edit tags' : 'Add tags' }}
          </button>
          <button @click.stop.prevent="handleDownload" class="transparent no-round left-align"><i class="small">download</i> Download ZIP</button>
          <li v-if="isOwner"><hr></li>
          <button v-if="isOwner" @click.stop.prevent="handleShare" class="transparent no-round left-align"><i class="small">person_add</i> Invite user</button>
          <button @click.stop.prevent="closeMenu(); isProjectSettingsModalOpen = true" class="transparent no-round left-align"><i class="small">settings</i> Project Settings</button>
          <li v-if="isOwner"><hr></li>
          <button v-if="isOwner" @click.stop.prevent="closeMenu(); isDeleteProjectModalOpen = true" class="transparent no-round left-align error-text"><i class="small">delete</i> Delete</button>
        </menu>
      </template>
    </FloatingElement>

    <!-- Share Project Token Dialog -->
    <dialog ref="shareDialog" popover>
      <p>Tell the invited person to go to this url:</p>
      <pre>{{ shareUrl }}</pre>
    </dialog>

    <!-- Tag Selection Modal -->
    <ProjectTagsModal
      v-model:open="isTagsModalOpen"
      :project-id="projectId"
    />

    <!-- Project Settings Modal -->
    <ProjectSettingsModal
      v-model:open="isProjectSettingsModalOpen"
      :project-id="projectId"
    />

    <!-- Delete Project Modal -->
    <ConfirmationModal
      v-model:open="isDeleteProjectModalOpen"
      title="Delete Project?"
      message="All project files will be permanently deleted."
      confirmLabel="Delete Project"
      cancelLabel="Cancel"
      variant="error"
      :onConfirm="handleDelete"
    />
  </div>
</template>

<style scoped>
.project-item {
  transition: all 0.2s ease-in-out;
  overflow: hidden;
  border-radius: 18px;
}
.project-item:hover {
  transform: translateY(-3px);
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 90%) !important;
}
.project-item.menu-active {
  transform: translateY(-3px);
  background-color: color-mix(in srgb, var(--inverse-surface), transparent 85%) !important;
}

.grid-cell {
  flex-direction: column;
  align-items: center;
  text-align: center;
}
.list-cell {
  flex-direction: row;
  align-items: center;
}

.grid-image {
  min-block-size: 6.5rem;
  max-block-size: 6.5rem;
}

.truncate-multi-line {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  overflow-wrap: anywhere;
  word-break: break-word;
  text-wrap: balance;
}

.project-link {
  width: 100%;
  height: 100%;
}

.project-text {
  min-width: 0;
  flex-shrink: 1;
}
.project-text > h5 {
  margin-bottom: 5px;
}
.list-cell .project-text {
  flex-grow: 3;
}
.grid-cell .project-text {
  width: 100%;
  margin-bottom: auto;
}

.list-cell .project-info {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  text-align: right;
  margin-right: 3rem;
  flex-grow: 1;
  flex-shrink: 1;
}
.grid-cell .project-info {
  margin-right: 3rem;
  justify-content: flex-end;
}

.list-cell .project-info .tag-container {
  justify-content: flex-end;
  width: 100%;
}

.tag, .edited-stamp {
  line-height: 1;
  padding: 8px;
  padding-top: 6px;
  padding-bottom: 4px;
  border-radius: 6px;
  text-wrap: balance;
}

.edited-stamp {
  margin-top: 10px;
  width: max-content;
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
.dropdown {
  box-shadow: none !important;
}
</style>
