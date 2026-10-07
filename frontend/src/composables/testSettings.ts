import { testSettingsConfig } from "@/config/testSettingsConfig";
import { useTestSettingsStore } from "@/stores/testSettings";
import { useSettings, type SettingsInstance } from "@/composables/settings";

export function useTestSettings(): SettingsInstance<typeof testSettingsConfig> {
  const store = useTestSettingsStore();
  return useSettings(testSettingsConfig, store);
}