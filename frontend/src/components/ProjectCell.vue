<script setup lang="ts">
import { ProjectSummary, formatTimeAgo, updateMetadata } from "@/services/managementApi";
import { View } from "@/services/projectController";
import FloatingElement from "@/components/FloatingElement.vue";
import { editCtrl } from "@/services/editorController";
import { ref, computed } from 'vue';
import TagList from '@/components/TagList.vue';
import ProjectTagsModal from "@/components/ProjectTagsModal.vue";

const props = defineProps<{
  project: ProjectSummary;
  view: View;
  availableTags?: string[];
}>();

const isMenuOpen = ref(false);
const isTagsModalOpen = ref(false);
const floatingElement = ref(); // Dropdown

const handleMenuToggle = (state: boolean) => {
  isMenuOpen.value = state;
};

function openTagsModal() {
  if (floatingElement.value) {
    floatingElement.value.isOpen = false;
  }
  isTagsModalOpen.value = true;
}

function handleDelete() {
  if (floatingElement.value) {
    floatingElement.value.isOpen = false;
  }
  emit('delete', props.project.id);
}

function handleShare() {
  if (floatingElement.value) {
    floatingElement.value.isOpen = false;
  }
  emit('share', props.project.id);
}

function handleDownload() {
  if (floatingElement.value) {
    floatingElement.value.isOpen = false;
  }
  emit('download', props.project.id);
}

function handleTagsUpdate(newTags: string[]) {
  if (!props.project.metadata) {
    props.project.metadata = {
      tags: [],
    };
  }
  props.project.metadata.tags = newTags;
  updateMetadata(props.project.id, props.project.metadata);
  emit('update-tags', { projectId: props.project.id, tags: newTags });
}

const hasTags = computed(() => props.project.metadata?.tags.length ?? 0 > 0);

const emit = defineEmits(['delete', 'share', 'download', 'update-tags']);
</script>

<template>
  <div :class="['surface-container project-item no-round padding', view === View.grid ? 'grid-cell' : 'list-cell', isMenuOpen ? 'menu-active' : '']">
    
    <RouterLink :to="`/projects/${project.id}`" class="project-link max relative no-gap" :class="{ 'row': view === View.list, 'field top-align left-align': view === View.grid }">
      
      <!--Project Icon-->
      <div v-if="view === View.list" class="shape secondary-container no-round small">
        <i class="medium">description</i>
      </div>
      <div v-if="view === View.grid" class="secondary-container no-round row center-align grid-image">
        <i class="extra">description</i>
      </div>

      <!--Project Title + Members-->
      <div class="project-text field">
        <h5 class="small truncate-multi-line bold">{{ project.title }}</h5>
        <TagList :tags="[project.owner.name, ...project.members.filter(m => m.id !== project.owner.id).map(m => m.name)]" :max-lines="1" :max-tags="1">
          <template #tag="{ tag }">
            <span class="grey-border" style="opacity: 0.8;">{{ tag }}</span>
          </template>
          <template #badge="{ count }">
            <span class="italic grey-text">+{{ count }}</span>
          </template>
        </TagList>
      </div>

      <!--Project Tags + Date-->
      <div class="project-info field">
        <TagList :tags="project.metadata?.tags ?? []" :max-lines="1" :max-tags="3" v-if="hasTags">
          <template #tag="{ tag }">
            <div class="secondary-container tag">{{ tag }}</div>
          </template>
          <template #badge="{ count }">
            <div class="surface-container-highest tag italic">+{{ count }}</div>
          </template>
        </TagList>
        <span class="surface-container-highest edited-stamp">{{ formatTimeAgo(project.updated_at) }} by {{ project.edited_by.name }}</span>
      </div>

    </RouterLink>

    <!--Dropdown-->
    <FloatingElement align="left" class="absolute right right-margin" :class="{ 'middle': view === View.list, 'bottom bottom-margin': view === View.grid }" @click.stop @toggle="handleMenuToggle" ref="floatingElement">
      <template #button>
        <button tabindex="0" class="square transparent more-button">
          <i>more_vert</i>
        </button>
      </template>
      <template #menu>
        <menu class="dropdown no-wrap no-round active surface-container shadow">
          <button @click.stop.prevent="openTagsModal" class="transparent no-round left-align">
            <i class="small">{{ hasTags ? 'label' : 'new_label' }}</i> {{ hasTags ? 'Edit tags' : 'Add tags' }}
          </button>
          <button @click.stop.prevent="handleDownload" class="transparent no-round left-align"><i class="small">download</i> Download ZIP</button>
          <li v-if="project.owner.id === editCtrl.user?.id"><hr></li>
          <button v-if="project.owner.id === editCtrl.user?.id" @click.stop.prevent="handleShare" class="transparent no-round left-align"><i class="small">person_add</i> Invite user</button>
          <button v-if="project.owner.id === editCtrl.user?.id" class="transparent no-round left-align"><i class="small">manage_accounts</i> View and remove users</button>
          <li v-if="project.owner.id === editCtrl.user?.id"><hr></li>
          <button v-if="project.owner.id === editCtrl.user?.id" @click.stop.prevent="handleDelete" class="transparent no-round left-align error-text"><i class="small">delete</i> Delete</button>
        </menu>
      </template>
    </FloatingElement>

    <!--Tag Selection Modal-->
    <ProjectTagsModal v-model="isTagsModalOpen" :project-name="project.title" :project-tags="project.metadata?.tags ?? []" :available-tags="availableTags" @update:tags="handleTagsUpdate"/>

  </div>
</template>

<style scoped>
.project-item {
  transition: all 0.2s ease-in-out;
  overflow: hidden;
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
</style>