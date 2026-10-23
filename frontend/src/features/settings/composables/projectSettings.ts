import { projectSettingsConfig } from "@/features/settings/configs/projectSettingsConfig";
import { useProjectSettingsStore } from "@/features/settings/stores/projectSettingsStore";
import { useSettings, type SettingsInstance } from "@/features/settings/composables/settings";
import { onMounted } from "vue";
import type { ProjectId } from "@/api/types";

export function useProjectSettings(projectId: ProjectId): SettingsInstance<typeof projectSettingsConfig, ProjectId> {
  const store = useProjectSettingsStore();

  onMounted(() => {
    store.fetchProjectSettings(projectId);
  });

  return useSettings(
    projectSettingsConfig,
    {
      getSetting: (key) => store.getSetting(projectId, key),
      setSetting: (key, value) => store.setSetting(projectId, key, value),
    },
    projectId // Context
  );
}
