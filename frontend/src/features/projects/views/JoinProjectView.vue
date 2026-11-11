<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useProjectsStore } from '@/shared/stores/projectsStore';
import { formatTimeAgo } from '@/shared/utils/formatters';
import TagList from '@/shared/components/composites/TagList.vue';
import type { JoinProjectPreview } from '@/api/types';

const route = useRoute();
const router = useRouter();
const projectsStore = useProjectsStore();

const token = route.params.token as string;

const preview = ref<JoinProjectPreview | null>(null);
const isLoading = ref(true);
const errorMessage = ref('');
const isAlreadyMember = ref(false);
const isJoining = ref(false);

onMounted(async () => {
  if (!token) {
    errorMessage.value = 'Invalid invitation link.';
    isLoading.value = false;
    return;
  }

  try {
    const res = await projectsStore.previewJoinProject({ token });
    if (res.success && res.data) {
      if (!projectsStore.state.isInitialized) {
        await projectsStore.fetchProjects();
      }
      if (projectsStore.getProjectById(res.data.id)) {
        isAlreadyMember.value = true;
        errorMessage.value = 'You are already a member of this project.';
      } else {
        preview.value = res.data;
      }
    } else {
      errorMessage.value = 'This invitation link is invalid or has expired.';
    }
  } catch {
    errorMessage.value = 'Failed to load project preview.';
  } finally {
    isLoading.value = false;
  }
});

async function handleJoin() {
  isJoining.value = true;
  errorMessage.value = '';
  try {
    const res = await projectsStore.joinProject({ token });
    if (res.success) {
      router.push({ name: 'projects' });
    } else {
      errorMessage.value = 'Failed to join project.';
    }
  } catch {
    errorMessage.value = 'Failed to join project.';
  } finally {
    isJoining.value = false;
  }
}

const ownerName = computed(() => {
  if (!preview.value) return '';
  return preview.value.owner.username;
});
</script>

<template>
  <main class="responsive no-padding no-margin max center-align middle-align">
    <!-- Loading Spinner -->
    <article v-if="isLoading" class="medium center-align box margin no-elevate surface-container error-card">
      <div class="content-box">
        <progress class="circle"></progress>
      </div>
    </article>

    <!-- Error -->
    <article v-else-if="errorMessage" class="medium center-align box margin no-elevate surface-container error-card">
      <div class="content-box">
        <i class="extra">error</i>
        <h5 class="bold responsive-title">{{ isAlreadyMember ? 'Already a member' : 'Invalid invitation' }}</h5>
        <h6 class="small responsive-description">{{ errorMessage }}</h6>
        <div class="empty-actions">
          <button tabindex="0" class="chip medium primary no-border" @click="router.push({ name: 'projects' })">
            <i>arrow_back</i> Go to Projects
          </button>
        </div>
      </div>
    </article>

    <!-- Join Project Cell -->
    <div v-else-if="preview" class="surface-container project-item padding grid-cell preview-grid-cell margin">
      <div class="project-link relative no-gap field top-align left-align">
        <!-- Project Icon -->
        <div class="secondary-container no-round row center-align grid-image">
          <i class="extra">description</i>
        </div>

        <!-- Project Title + Owner -->
        <div class="project-text field">
          <h5 class="small truncate-multi-line bold">{{ preview.title }}</h5>
          <TagList :tags="[ownerName]" :max-lines="1" :max-tags="1">
            <template #tag="{ tag }">
              <span class="grey-border" style="opacity: 0.8;">{{ tag }}</span>
            </template>
          </TagList>
        </div>

        <!-- Date Stamp -->
        <div class="project-info field">
          <span class="surface-container-highest edited-stamp">
            Updated {{ formatTimeAgo(preview.updatedAt) }}
          </span>
        </div>

        <!-- Actions -->
        <div class="join-action-wrapper">
          <button
            tabindex="0"
            class="primary no-border no-round join-button"
            :disabled="isJoining"
            @click="handleJoin"
          >
            <progress v-if="isJoining" class="circle small"></progress>
            <template v-else>
              <i>arrow_forward</i>
              <span>Join Project</span>
            </template>
          </button>

          <button
            tabindex="0"
            class="border no-round join-button"
            :disabled="isJoining"
            @click="router.push({ name: 'projects' })"
          >
            <span>Cancel</span>
          </button>
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.project-item {
  overflow: hidden;
  border-radius: 18px;
  box-sizing: border-box;
}

.preview-grid-cell {
  width: 100%;
  max-width: 380px;
}

.grid-cell {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.grid-image {
  min-block-size: 6.5rem;
  max-block-size: 6.5rem;
  width: 100%;
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
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
}

.project-text {
  min-width: 0;
  flex-shrink: 1;
  width: 100%;
  margin-bottom: auto;
}

.project-text > h5 {
  margin-bottom: 5px;
}

.project-info {
  justify-content: flex-end;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  width: 100%;
}

.edited-stamp {
  line-height: 1;
  padding: 8px;
  padding-top: 6px;
  padding-bottom: 4px;
  border-radius: 6px;
  text-wrap: balance;
  margin-top: 10px;
  width: max-content;
}

.join-action-wrapper {
  width: 100%;
  margin-top: 1.25rem;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.join-button {
  width: 100%;
  margin: 0 !important;
  box-sizing: border-box;
}

.error-card {
  padding: 2.5rem !important;
  border-radius: 18px !important;
}

.content-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.responsive-title {
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

.responsive-description {
  margin-top: 0;
  margin-bottom: 0;
}

.empty-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  justify-content: center;
  margin-top: 1.5rem;
}
</style>
