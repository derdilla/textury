import { accountSettingsConfig } from "@/features/settings/configs/accountSettingsConfig";
import { useAccountSettingsStore } from "@/features/settings/stores/accountSettingsStore";
import { useSettings, type SettingsInstance } from "@/features/settings/composables/settings";

export function useAccountSettings(): SettingsInstance<typeof accountSettingsConfig> {
  const store = useAccountSettingsStore();
  return useSettings(accountSettingsConfig, store);
}
