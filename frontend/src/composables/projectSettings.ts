import { projectSettingsConfig } from "@/config/projectSettingsConfig";
import { useProjectSettingsStore } from "@/stores/projectSettings";
import { useSettings, type SettingsInstance } from "@/composables/settings";

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