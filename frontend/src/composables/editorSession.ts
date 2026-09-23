import { reactive, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import * as Y from 'yjs'
import { ProjectTree } from '@/services/ProjectTree'
import { useProjectsStore } from '@/stores/projects'
import { API_ROOT } from '@/api/client'

function ySocketUrl(projectId: string, method = "edit") {
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${location.host}${API_ROOT}/project/${projectId}/${method}`;
}

const tree = reactive(new ProjectTree());

export const useEditorSession = () => {
  const route = useRoute()
  const router = useRouter()
  const projectsStore = useProjectsStore()

  const projectId = computed(() => route.params.id as string | undefined)
  const selectedPath = computed(() => (route.query.file as string) || null)
  const projectSummary = computed(() => projectId.value ? projectsStore.getProjectById(projectId.value) : undefined)

  function selectFile(path: string | null) {
    router.replace({ query: { ...route.query, file: path || undefined } })
  }

  function closeFile() {
    tree.openDocument = null;
    tree.openDocumentPath = null;
    tree.openDocumentUUID = null;
    selectFile(null);
  }

  watch(projectId, (newId) => {
      // If project is already loaded, do not reset.
      if (!newId || newId === tree.projectId) return;

      // If new project was opened, reset and load new project.
      tree.projectId = newId;
      tree.wsProvider?.dispose();
      closeFile();
      tree.updateProject(new Y.Doc(), ySocketUrl(newId));
    },
    { immediate: true }
  );

  watch(selectedPath, (newPath) => {
      if (!projectId.value) return;

      if (newPath) {
        // If selected file changed, open the new file.
        if (tree.openDocumentPath !== newPath) {
          tree.openFile(newPath);
        }
      } else if (tree.openDocumentPath) {
        // If project is still loaded, restore the selected file in the URL.
        selectFile(tree.openDocumentPath);
      }
    },
    { immediate: true }
  );

  return { projectId, selectedPath, projectSummary, tree, selectFile, closeFile }
}