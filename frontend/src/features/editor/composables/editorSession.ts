import { computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectsStore } from '@/shared/stores/projectsStore'
import { useEditorFileTree, type EditorFileTree, type FileMetadata } from '@/features/editor/composables/editorFileTree'
import type { Compiler } from '@/api/types'
import { useCompilerOutputStore } from '@/features/editor/stores/compilerOutputStore'
import { useLastOpenDocumentsStore } from '@/features/editor/stores/lastOpenDocumentStore'

let resetLogHandle: null | (() => void) = null;
let logHandle: null | (() => void) = null;
let setPdfHandle: null | (() => void) = null;

export const useEditorSession = () => {
  const route = useRoute()
  const router = useRouter()
  const projectsStore = useProjectsStore()

  const projectId = computed(() => route.params.id as string | undefined)
  const projectSummary = computed(() => projectId.value ? projectsStore.getProjectById(projectId.value) : undefined)
  const currentTree = computed<EditorFileTree | null>(() => {
    if (!projectId.value) return null;
    return useEditorFileTree(projectId.value);
  });
  const openDocumentUUID = computed(() => route.query.file?.toString() || null);
  const openDocument = computed((): Extract<FileMetadata, { kind: 'text' }> | null => {
    if (!openDocumentUUID.value) {
      console.info('No open file in path');
      return null;
    }
    const file = currentTree.value?.lookupEntryByUUID(openDocumentUUID.value);
    if (!file) {
      console.warn("Open file doesn't exist");
      closeFile();
      return null;
    }
    if (file.kind !== 'text') {
      console.warn('Invariant violated: non-text file opened');
      closeFile();
      return null;
    }
    return file;
  });

  const awareness = computed(() => currentTree.value?.ws.awareness);

  function selectFile(uuid: string | null) {
    router.replace({ query: { ...route.query, file: uuid || undefined } })
  }

  function closeFile() {
    selectFile(null);
  }

  // Restore last file opened in project
  const lastOpen = useLastOpenDocumentsStore();
  watch(projectId, (newProjectId) => {
    if (!newProjectId) return;
    const last = lastOpen.get(newProjectId);
    if (last) selectFile(last)
  }, { immediate: true });
  watch(openDocumentUUID, (newUUID) => {
    if (!projectId.value) return;
    lastOpen.update(projectId.value, newUUID);
  });

  function compile(compiler: Compiler) {
    if (!openDocument.value) return;
    currentTree.value?.ws.send({
      t: 'compile',
      uuid: openDocument.value.uuid,
      compiler,
    })
  }

  const outputStore = useCompilerOutputStore();
  watch(() => currentTree.value?.ws, (ws) => {
    outputStore.resetCompilerState();
    if (!ws) return;


    if (resetLogHandle) resetLogHandle();
    resetLogHandle = ws.on('resetLog', (msg) => {
      if (msg.uuid === openDocument.value?.uuid) {
        outputStore.setLog(null);
      }
    });
    if (logHandle) logHandle();
    logHandle = ws.on('log', (msg) => {
      if (msg.uuid === openDocument.value?.uuid) {
        let type: 'ok' | 'warn' | 'err' = 'ok';
        const lowercase = msg.data.toLowerCase();
        if (lowercase.indexOf('warn') !== -1) type = 'warn';
        if (lowercase.indexOf('err') !== -1) type = 'err';
        const existing = outputStore.state.log || "";
        outputStore.setLog(`${existing}\n${msg.data}`, type);
      }
    });
    if (setPdfHandle) setPdfHandle();
    setPdfHandle = ws.on('setPdf', (msg) => {
      if (msg.content && msg.uuid === openDocument.value?.uuid) {
        outputStore.setCompiledPdf(msg.content);
      }
    });
  }, { deep: true, immediate: true })


  return {
    projectId,
    openDocument,
    projectSummary,
    currentTree,
    selectFile,
    closeFile,
    awareness,
    compile,
  }
}
