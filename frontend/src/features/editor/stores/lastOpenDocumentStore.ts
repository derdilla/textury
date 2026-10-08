const lastOpenDocuments = new Map<String, string | null>();

function update(projectId: string, documentUUID: string | null) {
  lastOpenDocuments.set(projectId, documentUUID);
}

function get(projectId: string): string | null {
  return lastOpenDocuments.get(projectId) || null;
}

export function useLastOpenDocumentsStore() {
  return { update, get };
}
