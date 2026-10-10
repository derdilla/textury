import { useStorage } from '@vueuse/core';

export const useLastOpenDocumentsStore = () => {
  const lastOpenDocuments = useStorage<Record<string, string | null>>('last_open_documents', {});

  function update(projectId: string, documentUUID: string | null) {
    lastOpenDocuments.value[projectId] = documentUUID;
  }

  function get(projectId: string): string | null {
    return lastOpenDocuments.value[projectId] || null;
  }

  return { update, get };
}
