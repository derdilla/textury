import { reactive, computed } from 'vue';
import { api } from '@/api';
import type { PrivateAccountInfo, AdminUserInfo, UserInvite, UserId } from '@/api/types';

interface UsersManagementState {
  usersById: Record<UserId, PrivateAccountInfo>
  selectedAdminUser: AdminUserInfo | null
  isInitialized: boolean
  isLoading: boolean
}

const initialState: UsersManagementState = {
  usersById: {},
  selectedAdminUser: null,
  isInitialized: false,
  isLoading: false,
};

const state = reactive<UsersManagementState>({ ...initialState });

export const useUserManagementStore = () => {
  const users = computed(() => Object.values(state.usersById));

  async function fetchUsers() {
    state.isLoading = true;
    try {
      const res = await api.GET('/settings/users');
      if (!res.error && res.data) {
        const map: Record<UserId, PrivateAccountInfo> = {};
        const userList = res.data as PrivateAccountInfo[];
        for (const user of userList) {
          map[user.id] = user;
        }
        state.usersById = map;
      }
    } catch {

    } finally {
      state.isLoading = false;
      state.isInitialized = true;
    }
  }

  function getUserById(id: UserId): PrivateAccountInfo | undefined {
    return state.usersById[id];
  }

  async function fetchAdminUserDetails(id: UserId) {
    state.isLoading = true;
    try {
      const res = await api.GET('/settings/users/{id}', {
        params: { path: { id } },
      });
      if (!res.error && res.data) {
        const adminInfo = res.data as AdminUserInfo;
        state.selectedAdminUser = adminInfo;
        return { success: true, data: adminInfo };
      }
      return { success: false, error: res.error };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined };
    } finally {
      state.isLoading = false;
    }
  }

  function clearSelectedAdminUser() {
    state.selectedAdminUser = null;
  }

  async function generateInviteToken() {
    try {
      const res = await api.POST('/settings/users/invite');
      if (!res.error && res.data) {
        return { success: true, data: res.data as UserInvite };
      }
      return { success: false, error: '' };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined };
    }
  }

  async function deleteUser(id: UserId) {
    state.isLoading = true;
    try {
      const res = await api.DELETE('/settings/users/{id}', {
        params: { path: { id } },
        parseAs: 'text',
      });
      if (!res.error) {
        delete state.usersById[id];
        if (state.selectedAdminUser?.id === id) {
          state.selectedAdminUser = null;
        }
        return { success: true };
      }
      return { success: false, error: res.error };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined };
    } finally {
      state.isLoading = false;
    }
  }

  async function updateUsername(id: UserId, username: string) {
    try {
      const res = await api.PUT('/settings/users/{id}/username', {
        params: { path: { id } },
        body: username,
        parseAs: 'text',
      });
      if (!res.error) {
        if (state.usersById[id]) {
          Object.assign(state.usersById[id], { username });
        }
        return { success: true };
      }
      return { success: false, error: '' };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined };
    }
  }

  async function updateEmail(id: UserId, email: string) {
    try {
      const res = await api.PUT('/settings/users/{id}/email', {
        params: { path: { id } },
        body: email,
        parseAs: 'text',
      });
      if (!res.error) {
        if (state.usersById[id]) {
          Object.assign(state.usersById[id], { email });
        }
        return { success: true };
      }
      return { success: false, error: '' };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined };
    }
  }

  async function updateIsAdmin(id: UserId, isAdmin: boolean) {
    try {
      const res = await api.PUT('/settings/users/{id}/isAdmin', {
        params: { path: { id } },
        body: isAdmin,
        parseAs: 'text',
      });
      if (!res.error) {
        if (state.usersById[id]) {
          Object.assign(state.usersById[id], { isAdmin });
        }
        return { success: true };
      }
      return { success: false, error: '' };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : undefined };
    }
  }

  function clearUsers() {
    Object.assign(state, initialState);
  }

  return {
    state,
    users,
    fetchUsers,
    getUserById,
    fetchAdminUserDetails,
    clearSelectedAdminUser,
    generateInviteToken,
    deleteUser,
    updateUsername,
    updateEmail,
    updateIsAdmin,
    clearUsers,
  };
};
