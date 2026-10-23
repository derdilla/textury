import { reactive } from 'vue';
import { api } from '@/api';
import { projectSettingsConfig } from '@/features/settings/configs/projectSettingsConfig';
import { getDefaultsFromConfig } from '@/features/settings/utils/settingsDefaults';
import type { SettingsFromConfig } from '@/features/settings/utils/settingsSchema';
import { useProjectPublicShareStore } from '@/features/projects/stores/projectPublicShareStore';
import type { ProjectId } from '@/api/types';

// Extracted types related to the project settings.
export type ProjectSettings = SettingsFromConfig<typeof projectSettingsConfig>;
export type ProjectSettingKey = keyof ProjectSettings;

// Initially extract all defaults from the config.
const defaultsCache = getDefaultsFromConfig(projectSettingsConfig);

interface ProjectSettingsState {
  settingsByProject: Record<ProjectId, Partial<ProjectSettings>>;
  loadingProjects: Record<ProjectId, boolean>;
}

const initialState: ProjectSettingsState = {
  settingsByProject: {},
  loadingProjects: {},
};

const state = reactive<ProjectSettingsState>({ ...initialState });

export const useProjectSettingsStore = () => {
  // All project settings are currently server settings.
  async function fetchProjectSettings(projectId: ProjectId) {
    state.loadingProjects[projectId] = true;
    try {
      useProjectPublicShareStore(projectId).refresh();
    } catch {

    } finally {
      state.loadingProjects[projectId] = false;
    }
  }

  // GETTERS

  function getSetting<K extends ProjectSettingKey>(
    projectId: ProjectId,
    key: K
  ): ProjectSettings[K] {
    const projectMap = state.settingsByProject[projectId];
    return projectMap?.[key] ?? defaultsCache[key];
  }

  // SETTERS

  // Performing an optimistic update with potential rollback on error.
  async function setSetting<K extends ProjectSettingKey>(
    projectId: ProjectId,
    key: K,
    value: ProjectSettings[K]
  ): Promise<boolean> {
    if (!state.settingsByProject[projectId]) {
      state.settingsByProject[projectId] = {};
    }

    const previousValue = state.settingsByProject[projectId][key];
    if (previousValue === value) return true;

    state.settingsByProject[projectId][key] = value;

    try {
      // This cast is meassuringly needed since the api spec does not know that the expected parameter
      // always matches ProjectSettings because otherwise the server would be doing conflicting things.
      const res = await api.PUT(`/project/${projectId}/settings/${key}` as any, { body: value } as any);
      if (res.error) {
        state.settingsByProject[projectId][key] = previousValue;
        return false;
      }
      return true;
    } catch {
      state.settingsByProject[projectId][key] = previousValue;
      return false;
    }
  }

  function clearProjectSettings(projectId?: ProjectId) {
    if (projectId) {
      delete state.settingsByProject[projectId];
      delete state.loadingProjects[projectId];
    } else {
      Object.assign(state, initialState);
    }
  }

  return {
    state,
    fetchProjectSettings,
    getSetting,
    setSetting,
    clearProjectSettings,
  };
};
