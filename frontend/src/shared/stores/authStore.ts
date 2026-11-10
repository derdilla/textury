import { reactive, computed } from 'vue';
import router, { navigateToReturnPath, setReturnPath } from '@/router';
import { api } from '@/api';
import type {
  PrivateAccountInfo,
  LoginRequest,
  CreateAccountRequest,
  CreateSingleProjectAccountRequest,
} from '@/api/types';
import { useContactsStore } from '@/shared/stores/contactsStore';
import { useProjectsStore } from '@/shared/stores/projectsStore';
import { useAccountSettingsStore } from '@/features/settings/stores/accountSettingsStore';
import { useProjectSettingsStore } from '@/features/settings/stores/projectSettingsStore';
import { useTestSettingsStore } from '@/features/settings/stores/testSettingsStore';
import { useUserManagementStore } from '@/features/settings/stores/userManagementStore';
import { useProjectMembersStore } from '@/features/projects/stores/projectMembersStore';

interface AuthState {
  user: PrivateAccountInfo | null
  isInitialized: boolean
  isLoading: boolean
}

const initialState: AuthState = {
  user: null,
  isInitialized: false,
  isLoading: false,
}

const state = reactive<AuthState>({ ...initialState })

const authChannel = typeof window !== 'undefined' ? new BroadcastChannel('auth_sync_channel') : null;

export const useAuthStore = () => {
  const isAuthenticated = computed(() => state.user != null);

  if (authChannel && !authChannel.onmessage) {
    authChannel.onmessage = async (event) => {
      if (event.data === 'LOGOUT') {
        if (state.user != null) {
          if (router.currentRoute.value.meta.requiresAuth) {
            setReturnPath(router.currentRoute.value.fullPath);
          }
          clearEverything();
          state.isInitialized = false;
          await router.push({ name: 'login' });
        }
      } else if (event.data === 'LOGIN') {
        if (state.user == null) {
          await checkAuth();
          await navigateToReturnPath();
        }
      }
    };
  }

  function broadcastAuthEvent(event: 'LOGIN' | 'LOGOUT') {
    authChannel?.postMessage(event);
  }

  async function loadUserSessionData() {
    await Promise.all([
      useContactsStore().fetchContacts(),
      useProjectsStore().fetchProjects(),
      useAccountSettingsStore().fetchServerSettings(),
    ]);
  }

  async function checkAuth() {
    state.isLoading = true;
    try {
      const res = await api.GET('/user');
      if (!res.error && res.data) {
        state.user = res.data;
        await loadUserSessionData();
      } else {
        state.user = null;
      }
    } catch {
      state.user = null;
    } finally {
      state.isLoading = false;
      state.isInitialized = true;
    }
  }

  async function login(data: LoginRequest) {
    state.isLoading = true;
    try {
      const res = await api.POST('/user/login', { body: data });
      if (res.data) {
        state.user = res.data;
        await loadUserSessionData();
        broadcastAuthEvent('LOGIN');
        await navigateToReturnPath();
        return { success: true };
      }
      return {
        success: false,
        error: res.error,
      };
    } catch (err) {
      return {
        success: false,
        error: err instanceof Error ? err.message : undefined,
      };
    } finally {
      state.isLoading = false;
    }
  }

  async function register(data: CreateAccountRequest) {
    state.isLoading = true;
    try {
      const res = await api.POST('/user', { body: data });
      if (res.data) {
        state.user = res.data;
        await loadUserSessionData();
        broadcastAuthEvent('LOGIN');
        await navigateToReturnPath();
        return { success: true };
      }
      return {
        success: false,
        error: res.error,
      };
    } catch (err) {
      return {
        success: false,
        error: err instanceof Error ? err.message : undefined,
      };
    } finally {
      state.isLoading = false;
    }
  }

  async function joinProject(data: CreateSingleProjectAccountRequest) {
    state.isLoading = true;
    try {
      const res = await api.POST('/user/singleProjectUser', { body: data });
      if (res.data) {
        await checkAuth();
        broadcastAuthEvent('LOGIN');
        await router.push(`/projects/${res.data.projectId}`);
        return { success: true };
      }
      return {
        success: false,
        error: res.error,
      };
    } catch (err) {
      return {
        success: false,
        error: err instanceof Error ? err.message : undefined,
      };
    } finally {
      state.isLoading = false;
    }
  }

  async function logout() {
    state.isLoading = true;
    try {
      await api.POST('/user/logout');
    } catch {

    } finally {
      clearEverything();
      state.isInitialized = false;
      broadcastAuthEvent('LOGOUT');
      await router.push({ name: 'login' });
      state.isLoading = false;
    }
  }

  async function deleteAccount() {
    try {
      const res = await api.DELETE('/user');
      if (!res.error) {
        clearEverything();
        state.isInitialized = false;
        broadcastAuthEvent('LOGOUT');
        await router.push({ name: 'login' });
        return { success: true };
      }
      return {
        success: false,
        error: '',
      };
    } catch (err) {
      return {
        success: false,
        error: err instanceof Error ? err.message : undefined,
      };
    }
  }

  async function updateUsername(username: string) {
    try {
      const res = await api.PUT('/user/username', { body: username });
      if (!res.error) {
        if (state.user) {
          state.user.username = username;
        }
        return { success: true };
      }
      return {
        success: false,
        error: ''
      };
    } catch (err) {
      return {
        success: false,
        error: err instanceof Error ? err.message : undefined,
      };
    }
  }

  async function updateEmail(email: string) {
    try {
      const res = await api.PUT('/user/email', { body: email });
      if (!res.error) {
        if (state.user) {
          state.user.email = email;
        }
        return { success: true };
      }
      return {
        success: false,
        error: ''
      };
    } catch (err) {
      return {
        success: false,
        error: err instanceof Error ? err.message : undefined,
      };
    }
  }

  function clearEverything() {
    state.user = null;
    useContactsStore().clearContacts();
    useProjectsStore().clearProjects();
    useAccountSettingsStore().clearServerSettings();
    useProjectSettingsStore().clearProjectSettings();
    useProjectMembersStore().clearMembers();
    useTestSettingsStore().clearTestSettings();
    useUserManagementStore().clearUsers();
  }

  return {
    state,
    isAuthenticated,
    checkAuth,
    login,
    register,
    joinProject,
    logout,
    deleteAccount,
    updateUsername,
    updateEmail,
  }
}
