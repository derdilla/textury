import { reactive, computed } from 'vue';
import router from '@/router';
import { api } from '@/api/api';
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

export const useAuthStore = () => {
  const isAuthenticated = computed(() => state.user != null);

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
        await router.push({ name: 'projects' });
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
        await router.push({ name: 'projects' });
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
      state.isLoading = false;
      await router.push({ name: 'login' });
    }
  }

  async function deleteAccount() {
    try {
      const res = await api.DELETE('/user');
      if (!res.error) {
        clearEverything();
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
    useTestSettingsStore().clearTestSettings();
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
    updateEmail
  }
}
