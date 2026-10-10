import { reactive, computed } from 'vue'
import { api } from '@/api'
import type {
  ProjectSummary,
  UserMeta,
  JoinProjectRequest,
  ProjectInvite,
} from '@/api/types'
import { useProjectSettingsStore } from '@/features/settings/stores/projectSettingsStore'
import { useEditorFileTree } from '@/features/editor/composables/editorFileTree'
import { type ProjectTemplateOption } from "@/features/projects/stores/projectsLayoutStore";
import { useLastOpenDocumentsStore } from '@/features/editor/stores/lastOpenDocumentStore'
import router from "@/router";

interface ProjectsState {
  byProjectId: Record<string, ProjectSummary>
  isInitialized: boolean
  isLoading: boolean
}

const initialState: ProjectsState = {
  byProjectId: {},
  isInitialized: false,
  isLoading: false,
}

const state = reactive<ProjectsState>({ ...initialState })

export const useProjectsStore = () => {
  const projects = computed(() => Object.values(state.byProjectId))

  async function fetchProjects() {
    state.isLoading = true
    try {
      const res = await api.GET('/projects/index')
      if (!res.error && res.data) {
        const map: Record<string, ProjectSummary> = {}
        const projects = res.data as unknown as ProjectSummary[]
        for (const project of projects) {
          map[project.id] = project
        }
        state.byProjectId = map
      }
    } catch {

    } finally {
      state.isLoading = false
      state.isInitialized = true
    }
  }

  function getProjectById(id: string): ProjectSummary | undefined {
    return state.byProjectId[id]
  }

  async function createProject(title: string, template: ProjectTemplateOption) {
    state.isLoading = true
    try {
      const res = await api.POST('/projects/create', {
        body: {
          title,
        },
        parseAs: 'text',
      })
      if (!res.error) {
        await fetchProjects()
        const projectId = JSON.parse(res.data ?? "");
        const editorFileTree = useEditorFileTree(projectId);
        const lastOpenDocuments = useLastOpenDocumentsStore();
        if (template == 'latex') {
          const response = await fetch('/samples/main.tex');
          const content = await response.text();
          editorFileTree.root.value?.createTextFile("main.tex", content);
          const fileId = editorFileTree.lookupEntryByPath("/main.tex")?.uuid;
          lastOpenDocuments.update(projectId, fileId ?? null);
        }
        if (template == 'typst') {
          const response = await fetch('/samples/main.typ');
          const content = await response.text();
          editorFileTree.root.value?.createTextFile("main.typ", content);
          const fileId = editorFileTree.lookupEntryByPath("/main.typ")?.uuid;
          lastOpenDocuments.update(projectId, fileId ?? null);
        }
        router.push(`projects/${projectId}`)
        return { success: true }
      }
      return { success: false, error: res.error }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined }
    } finally {
      state.isLoading = false
    }
  }

  async function updateTitle(id: string, title: string) {
    try {
      const res = await api.PUT('/project/{id}/title', {
        params: { path: { id } },
        body: title,
        parseAs: 'text',
      })
      if (!res.error) {
        if (state.byProjectId[id]) {
          Object.assign(state.byProjectId[id], { title })
        }
        return { success: true }
      }
      return { success: false, error: '' }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined }
    }
  }

  async function deleteProject(id: string) {
    state.isLoading = true
    try {
      const res = await api.DELETE('/project/{id}', {
        params: { path: { id } },
        parseAs: 'text',
      })
      if (!res.error) {
        delete state.byProjectId[id]
        useProjectSettingsStore().clearProjectSettings(id)
        return { success: true }
      }
      return { success: false, error: res.error }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined }
    } finally {
      state.isLoading = false
    }
  }

  async function joinProject(data: JoinProjectRequest) {
    state.isLoading = true
    try {
      const res = await api.POST('/projects/join', {
        body: data,
        parseAs: 'text',
      })
      if (!res.error) {
        await fetchProjects()
        return { success: true }
      }
      return { success: false, error: res.error }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined }
    } finally {
      state.isLoading = false
    }
  }

  async function createShareToken(id: string) {
    try {
      const res = await api.POST('/project/{id}/share', {
        params: { path: { id } },
      })
      if (res.data) {
        return { success: true, data: res.data as ProjectInvite }
      }
      return { success: false, error: res.error }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined }
    }
  }

  async function updateMetadata(id: string, metadata: UserMeta) {
    try {
      const res = await api.PUT('/project/{id}/meta', {
        params: { path: { id } },
        body: metadata as unknown as Record<string, unknown>,
        parseAs: 'text',
      })
      if (!res.error) {
        if (state.byProjectId[id]) {
          Object.assign(state.byProjectId[id], { metadata })
        }
        return { success: true }
      }
      return { success: false, error: '' }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined }
    }
  }

  function clearProjects() {
    Object.assign(state, initialState)
  }

  return {
    state,
    projects,
    fetchProjects,
    getProjectById,
    createProject,
    updateTitle,
    deleteProject,
    joinProject,
    createShareToken,
    updateMetadata,
    clearProjects,
  }
}
