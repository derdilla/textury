import { reactive, computed } from 'vue'
import { api } from '@/api'
import type {
  ProjectSummary,
  UserMeta,
  JoinProjectRequest,
  ProjectInvite,
  ProjectId,
  ProjectPermissionLevel,
  JoinProjectPreview,
} from '@/api/types'
import { useProjectSettingsStore } from '@/features/settings/stores/projectSettingsStore'
import { useEditorFileTree } from '@/features/editor/composables/editorFileTree'
import { type ProjectTemplateOption } from "@/features/projects/stores/projectsLayoutStore";
import { useLastOpenDocumentsStore } from '@/features/editor/stores/lastOpenDocumentStore'
import router from "@/router";
import { useProjectMembersStore } from '@/features/projects/stores/projectMembersStore'
import { useContactsStore } from '@/shared/stores/contactsStore';

interface ProjectsState {
  byProjectId: Record<ProjectId, ProjectSummary>
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
        const map: Record<ProjectId, ProjectSummary> = {}
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

  function getProjectById(id: ProjectId): ProjectSummary | undefined {
    return state.byProjectId[id]
  }

  function getPermissionLevel(id: ProjectId): ProjectPermissionLevel | undefined {
    return state.byProjectId[id]?.permissions
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

  async function updateTitle(id: ProjectId, title: string) {
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

  async function deleteProject(id: ProjectId) {
    state.isLoading = true
    try {
      const res = await api.DELETE('/project/{id}', {
        params: { path: { id } },
        parseAs: 'text',
      })
      if (!res.error) {
        clearEverythingOfProject(id)
        router.push('/projects')
        return { success: true }
      }
      return { success: false, error: res.error }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined }
    } finally {
      state.isLoading = false
    }
  }

  async function leaveProject(id: ProjectId) {
    state.isLoading = true
    try {
      const res = await api.POST('/project/{id}/leave', {
        params: { path: { id } },
        parseAs: 'text',
      })
      if (!res.error) {
        clearEverythingOfProject(id)
        router.push('/projects')
        return { success: true }
      }
      return { success: false, error: res.error }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined }
    } finally {
      state.isLoading = false
    }
  }

  async function previewJoinProject(data: JoinProjectRequest) {
    try {
      const res = await api.POST('/projects/preview', {
        body: data,
      })
      if (!res.error && res.data) {
        return { success: true, data: res.data as JoinProjectPreview }
      }
      return { success: false, error: res.error }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined }
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
        await useContactsStore().fetchContacts()
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

  async function createShareToken(id: ProjectId, initialPermissionLevel: ProjectPermissionLevel) {
    try {
      const res = await api.POST('/project/{id}/share', {
        params: { path: { id } },
        body: initialPermissionLevel
      })
      if (res.data) {
        return { success: true, data: res.data as ProjectInvite }
      }
      return { success: false, error: res.error }
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined }
    }
  }

  async function updateMetadata(id: ProjectId, metadata: UserMeta) {
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

  function clearEverythingOfProject(id: ProjectId) {
    delete state.byProjectId[id]
    useProjectSettingsStore().clearProjectSettings(id)
    useProjectMembersStore().clearMembers(id)
  }

  function clearProjects() {
    Object.assign(state, initialState)
  }

  return {
    state,
    projects,
    fetchProjects,
    getProjectById,
    getPermissionLevel,
    createProject,
    updateTitle,
    deleteProject,
    leaveProject,
    previewJoinProject,
    joinProject,
    createShareToken,
    updateMetadata,
    clearEverythingOfProject,
    clearProjects,
  }
}
