import { reactive } from 'vue';
import { api } from '@/api';
import type { ProjectId, UserId, ProjectPermissionLevel, ProjectMemberInfo } from '@/api/types';
import { useProjectsStore } from '@/shared/stores/projectsStore';
import { useAuthStore } from '@/shared/stores/authStore';

interface ProjectMembersState {
  membersByProject: Record<ProjectId, Record<UserId, ProjectMemberInfo>>;
  isInitializedByProject: Record<ProjectId, boolean>;
  loadingProjects: Record<ProjectId, boolean>;
}

const initialState: ProjectMembersState = {
  membersByProject: {},
  isInitializedByProject: {},
  loadingProjects: {},
};

const state = reactive<ProjectMembersState>({ ...initialState });

export const useProjectMembersStore = () => {
  function getMembers(projectId: ProjectId): ProjectMemberInfo[] {
    return Object.values(state.membersByProject[projectId] ?? {});
  }

  function isInitialized(projectId: ProjectId): boolean {
    return !!state.isInitializedByProject[projectId];
  }

  function isLoading(projectId: ProjectId): boolean {
    return !!state.loadingProjects[projectId];
  }

  async function fetchMembers(projectId: ProjectId) {
    state.loadingProjects[projectId] = true;
    try {
      const res = await api.GET('/project/{id}/members', {
        params: { path: { id: projectId } },
      });
      if (!res.error && res.data) {
        const map: Record<UserId, ProjectMemberInfo> = {};
        const memberList = res.data as { id: UserId; permissions: ProjectPermissionLevel }[];

        for (const member of memberList) {
          map[member.id] = {
            id: member.id,
            permissions: member.permissions,
          };
        }
        state.membersByProject[projectId] = map;
        state.isInitializedByProject[projectId] = true;
      }
    } catch {
    } finally {
      state.loadingProjects[projectId] = false;
    }
  }

  async function kickMember(projectId: ProjectId, userId: UserId) {
    state.loadingProjects[projectId] = true;
    try {
      const res = await api.POST('/project/{pid}/members/{uid}/kick', {
        params: { path: { pid: projectId, uid: userId } },
        parseAs: 'text',
      });
      if (!res.error) {
        if (state.membersByProject[projectId]) {
          delete state.membersByProject[projectId][userId];
        }
        return { success: true };
      }
      return { success: false, error: res.error };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined };
    } finally {
      state.loadingProjects[projectId] = false;
    }
  }

  async function transferOwnership(projectId: ProjectId, userId: UserId) {
    try {
      const res = await api.POST('/project/{pid}/members/{uid}/transferOwnership', {
        params: { path: { pid: projectId, uid: userId } },
        parseAs: 'text',
      });
      if (!res.error) {
        const project = useProjectsStore().getProjectById(projectId);
        if (project) {
          project.owner = userId;
        }
        return { success: true };
      }
      return { success: false, error: res.error };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined };
    }
  }

  async function updatePermissionLevel(projectId: ProjectId, userId: UserId, permissionLevel: ProjectPermissionLevel) {
    try {
      const res = await api.PUT('/project/{pid}/members/{uid}/permissions', {
        params: { path: { pid: projectId, uid: userId } },
        body: permissionLevel,
        parseAs: 'text',
      });
      if (!res.error) {
        if (state.membersByProject[projectId]?.[userId]) {
          state.membersByProject[projectId][userId].permissions = permissionLevel;
        }
        if (useAuthStore().state.user?.id === userId) {
          const project = useProjectsStore().getProjectById(projectId);
          if (project) {
            project.permissions = permissionLevel;
          }
        }

        return { success: true };
      }
      return { success: false, error: '' };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined };
    }
  }

  function clearMembers(projectId?: ProjectId) {
    if (projectId) {
      delete state.membersByProject[projectId];
      delete state.isInitializedByProject[projectId];
      delete state.loadingProjects[projectId];
    } else {
      Object.assign(state, initialState);
    }
  }

  return {
    state,
    getMembers,
    isInitialized,
    isLoading,
    fetchMembers,
    kickMember,
    transferOwnership,
    updatePermissionLevel,
    clearMembers,
  };
};
