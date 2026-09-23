import { accountSettingsConfig } from "@/config/accountSettingsConfig";
import { testSettingsConfig } from "@/config/testSettingsConfig";
import { projectSettingsConfig } from "@/config/projectSettingsConfig";
import { createSettingsStore } from "@/config/settingsEngine";
import { type SettingsStore } from "@/config/settingsEngine";
import type { ProjectSummary } from "@/api/types";
import { useProjectsStore } from "@/stores/projects";

export const accountSettingsStore: SettingsStore<typeof accountSettingsConfig> = createSettingsStore(accountSettingsConfig);

export const testSettingsStore: SettingsStore<typeof testSettingsConfig> = createSettingsStore(testSettingsConfig);

const projectSettingsStores = new Map<string, SettingsStore<typeof projectSettingsConfig, ProjectSummary>>();

export function getProjectSettingsStore(projectId: string): SettingsStore<typeof projectSettingsConfig, ProjectSummary> {
  if (!projectSettingsStores.has(projectId)) {
    const project = useProjectsStore().getProjectById(projectId);
    const store = createSettingsStore(projectSettingsConfig, project);
    projectSettingsStores.set(projectId, store);
  }
  return projectSettingsStores.get(projectId)!;
}

export function removeProjectSettingsStore(projectId: string) {
  projectSettingsStores.delete(projectId);
}