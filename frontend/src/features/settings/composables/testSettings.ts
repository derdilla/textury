import { testSettingsConfig } from "@/features/settings/configs/testSettingsConfig";
import { useTestSettingsStore } from "@/features/settings/stores/testSettingsStore";
import { useSettings, type SettingsInstance } from "@/features/settings/composables/settings";

export function useTestSettings(): SettingsInstance<typeof testSettingsConfig> {
  const store = useTestSettingsStore();
  return useSettings(testSettingsConfig, store);
}
