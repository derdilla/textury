import type { ProjectId } from '@/api/types';
import { useStorage } from '@vueuse/core';

export const useLastOpenDocumentsStore = () => {
  const lastOpenDocuments = useStorage<Record<ProjectId, string | null>>('last_open_documents', {});

  function update(projectId: ProjectId, documentUUID: string | null) {
    lastOpenDocuments.value[projectId] = documentUUID;
  }

  function get(projectId: ProjectId): string | null {
    return lastOpenDocuments.value[projectId] || null;
  }

  return { update, get };
}
