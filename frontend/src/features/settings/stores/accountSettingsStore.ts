import { reactive, computed } from 'vue';
import { useAuthStore } from '@/shared/stores/authStore';
import { api } from '@/api/api';
import type { ServerSettings } from '@/api/types';
import { accountSettingsConfig } from '@/features/settings/configs/accountSettingsConfig';
import { getDefaultsFromConfig } from '@/features/settings/utils/settingsDefaults';
import type { SettingsFromConfig } from '@/features/settings/utils/settingsSchema';

// Extracted types related to the account settings.
export type AccountSettings = SettingsFromConfig<typeof accountSettingsConfig>;
export type AccountSettingKey = keyof AccountSettings;

// Initially extract all defaults from the config.
// They are only used one-time when a client setting is accessed that did not exist before.
const defaultsCache = getDefaultsFromConfig(accountSettingsConfig);

interface AccountSettingsState {
  serverSettings: ServerSettings | null;
  isLoading: boolean;
  isInitialized: boolean;
}

const initialState: AccountSettingsState = {
  serverSettings: null,
  isLoading: false,
  isInitialized: false,
}

const state = reactive<AccountSettingsState>({ ...initialState })

export const useAccountSettingsStore = () => {
  const authStore = useAuthStore();
  const clientSettings = computed(() => authStore.state.user?.settings ?? {});

  // Client settings are already fetched with the auth data.
  async function fetchServerSettings() {
    state.isLoading = true;
    try {
      const res = await api.GET('/settings');
      if (!res.error && res.data) {
        state.serverSettings = res.data;
      }
    } catch {
      state.serverSettings = null;
    } finally {
      state.isLoading = false;
      state.isInitialized = true;
    }
  }

  // GETTERS

  function _getServerSetting<K extends keyof ServerSettings>(key: K): ServerSettings[K] {
    return (state.serverSettings?.[key] ?? defaultsCache[key]) as ServerSettings[K];
  }

  function _getClientSetting<K extends AccountSettingKey>(key: K): AccountSettings[K] {
    const user = authStore.state.user;
    const value = clientSettings.value[key] ?? defaultsCache[key];

    // If the client setting does not exist yet we need to create it.
    // After the first time the server would have already send the correct data.
    if (user && clientSettings.value[key] === undefined && value !== undefined) {
      _setClientSetting(key, value);
    }
    return value as AccountSettings[K];
  }

  // Handles client and server settings transparently.
  function getSetting<K extends AccountSettingKey>(key: K): AccountSettings[K] {
    if (state.serverSettings && key in state.serverSettings) {
      // This cast should never fail except if the config and the openapi spec for server settings deviates.
      return _getServerSetting(key as any) as AccountSettings[K];
    }
    return _getClientSetting(key);
  }

  // SETTERS

  // Performing an optimistic update with potential rollback on error.
  async function _setServerSetting<K extends keyof ServerSettings>(
    key: K,
    value: ServerSettings[K]
  ): Promise<boolean> {
    if (!state.serverSettings) return false;

    const previousValue = state.serverSettings[key];
    if (previousValue === value) return true;

    state.serverSettings[key] = value;

    try {
      // This cast is sadly needed since the api spec does not know that the expected parameter
      // always matches ServerSettings because otherwise the server would be doing conflicting things.
      const res = await api.PUT(`/settings/${key}`, { body: value } as any);
      if (res.error) {
        state.serverSettings[key] = previousValue;
        return false;
      }
      return true;
    } catch {
      state.serverSettings[key] = previousValue;
      return false;
    }
  }

  // Performing an optimistic update with potential rollback on error.
  async function _setClientSetting<K extends AccountSettingKey>(
    key: K,
    value: AccountSettings[K]
  ): Promise<boolean> {
    const user = authStore.state.user;
    if (!user) return false;

    const currentSettings = user.settings ?? {};
    const previousValue = currentSettings[key];
    if (previousValue === value) return true;

    user.settings = { ...currentSettings, [key]: value };

    try {
      const res = await api.PUT('/user/clientSettings', { body: user.settings });
      if (res.error) {
        user.settings[key] = previousValue;
        return false;
      }
      return true;
    } catch {
      user.settings[key] = previousValue;
      return false;
    }
  }

  // Handles client and server settings transparently.
  async function setSetting<K extends AccountSettingKey>(
    key: K,
    value: AccountSettings[K]
  ): Promise<boolean> {
    if (state.serverSettings && key in state.serverSettings) {
      // This cast should never fail except if the config and the openapi spec for server settings deviates.
      return _setServerSetting(key as any, value as any);
    }
    return _setClientSetting(key, value);
  }

  function clearServerSettings() {
    state.serverSettings = null;
    state.isInitialized = false;
    state.isLoading = false;
  }

  return {
    state,
    fetchServerSettings,
    getSetting,
    setSetting,
    clearServerSettings,
  };
};
