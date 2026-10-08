import { projectSettingsConfig } from "@/features/settings/configs/projectSettingsConfig";
import { useProjectSettingsStore } from "@/features/settings/stores/projectSettingsStore";
import { useSettings, type SettingsInstance } from "@/features/settings/composables/settings";

export function useProjectSettings(projectId: string): SettingsInstance<typeof projectSettingsConfig, string> {
  const store = useProjectSettingsStore();
  store.fetchProjectSettings(projectId);

  return useSettings(
    projectSettingsConfig,
    {
      getSetting: (key) => store.getSetting(projectId, key),
      setSetting: (key, value) => store.setSetting(projectId, key, value),
    },
    projectId // Context
  );
}
