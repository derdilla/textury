import { reactive, computed } from 'vue';
import router from '@/router';
import { api } from '@/api/client';
import type {
  PrivateAccountInfo,
  LoginRequest,
  CreateAccountRequest,
} from '@/api/types';
import { useContactsStore } from '@/stores/contacts';
import { useProjectsStore } from '@/stores/projects';

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

  async function logout() {
    state.isLoading = true;
    try {
      await api.POST('/user/logout');
    } catch {
      
    } finally {
      state.user = null;
      state.isLoading = false;
      useContactsStore().clearContacts();
      useProjectsStore().clearProjects();
      await router.push({ name: 'login' });
    }
  }

  return {
    state,
    isAuthenticated,
    checkAuth,
    login,
    register,
    logout,
  }
}