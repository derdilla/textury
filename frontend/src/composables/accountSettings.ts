import { accountSettingsConfig } from "@/config/accountSettingsConfig";
import { useAccountSettingsStore } from "@/stores/accountSettings";
import { useSettings, type SettingsInstance } from "@/composables/settings";

export function useAccountSettings(): SettingsInstance<typeof accountSettingsConfig> {
  const store = useAccountSettingsStore();
  return useSettings(accountSettingsConfig, store);
}