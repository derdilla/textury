import { reactive } from 'vue';
import { testSettingsConfig } from '@/config/testSettingsConfig';
import { getDefaultsFromConfig } from '@/components/settings/settingsDefaults';
import type { SettingsFromConfig } from '@/components/settings/settingsSchema';

export type TestSettings = SettingsFromConfig<typeof testSettingsConfig>;
export type TestSettingKey = keyof TestSettings;

const defaultsCache = getDefaultsFromConfig(testSettingsConfig);

const state = reactive<Record<string, any>>({ ...defaultsCache });

export const useTestSettingsStore = () => {
  function getSetting<K extends TestSettingKey>(key: K): TestSettings[K] {
    return (state[key as string] ?? defaultsCache[key as string]) as TestSettings[K];
  }

  function setSetting<K extends TestSettingKey>(key: K, value: TestSettings[K]): boolean {
    state[key as string] = value;
    return true;
  }

  function clearTestSettings() {
    Object.assign(state, getDefaultsFromConfig(testSettingsConfig));
  }

  return {
    state,
    getSetting,
    setSetting,
    clearTestSettings,
  };
};