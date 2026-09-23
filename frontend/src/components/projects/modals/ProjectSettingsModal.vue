<script setup lang="ts">
import { computed } from "vue";
import ModalPopover from "@/components/general/ModalPopover.vue";
import EmbeddedRouter from "@/components/general/EmbeddedRouter.vue";
import SettingsView from "@/components/settings/SettingsView.vue";
import { projectSettingsConfig } from "@/config/projectSettingsConfig";
import { getProjectSettingsStore } from "@/config/settingsStore";
import { useProjectsStore } from "@/stores/projects";
import type { RouteRecordRaw } from "vue-router";

const open = defineModel<boolean>("open", { default: false });

const props = defineProps<{
  projectId: string;
}>();

const projectsStore = useProjectsStore();

const project = computed(() => projectsStore.getProjectById(props.projectId));

const currentStore = computed(() => getProjectSettingsStore(props.projectId));

const embeddedRoutes: RouteRecordRaw[] = [
  {
    path: "/settings/:tabId?/:pageId*",
    component: SettingsView,
  },
];
</script>

<template>
  <ModalPopover
    v-model:open="open"
    max-width="900px"
    :no-padding="true"
  >
    <div class="project-settings-content">
      <EmbeddedRouter
        v-if="open"
        :routes="embeddedRoutes"
        initial-route="/settings"
      >
        <SettingsView
          :config="projectSettingsConfig"
          :store="currentStore"
          :embedded="true"
          title="Project Settings"
          :subtitle="project?.title"
        />
      </EmbeddedRouter>
    </div>
  </ModalPopover>
</template>

<style scoped>
.project-settings-content {
  width: 100%;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>