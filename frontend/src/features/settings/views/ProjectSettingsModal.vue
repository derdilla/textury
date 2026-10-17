<script setup lang="ts">
import { computed } from "vue";
import ModalPopover from "@/shared/components/primitives/ModalPopover.vue";
import EmbeddedRouter from "@/shared/components/layouts/EmbeddedRouter.vue";
import SettingsView from "@/features/settings/views/SettingsView.vue";
import { projectSettingsConfig } from "@/features/settings/configs/projectSettingsConfig";
import { useProjectSettings } from "@/features/settings/composables/projectSettings";
import { useProjectsStore } from "@/shared/stores/projectsStore";
import type { RouteRecordRaw } from "vue-router";

const open = defineModel<boolean>("open", { default: false });

const props = defineProps<{
  projectId: string;
}>();

const projectsStore = useProjectsStore();

const project = computed(() => projectsStore.getProjectById(props.projectId));

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
          :instance="useProjectSettings(props.projectId)"
          :embedded="true"
          title="Project Settings"
          :subtitle="project?.title"
          pageId="project_settings"
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
