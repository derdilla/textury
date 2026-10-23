import { reactive, computed } from 'vue'
import { api } from '@/api'
import type { User, UserId } from '@/api/types'
import { useAuthStore } from '@/shared/stores/authStore'

interface ContactsState {
  byUserId: Record<UserId, User>
  isInitialized: boolean
  isLoading: boolean
}

const initialState: ContactsState = {
  byUserId: {},
  isInitialized: false,
  isLoading: false,
}

const state = reactive<ContactsState>({ ...initialState })

export const useContactsStore = () => {
  const authStore = useAuthStore();

  const contactsList = computed(() => {
    const list = Object.values(state.byUserId);
    const currentUser = authStore.state.user;
    if (currentUser && !state.byUserId[currentUser.id]) {
      return [currentUser, ...list];
    }
    return list;
  });

  async function fetchContacts() {
    state.isLoading = true;
    try {
      const res = await api.GET('/contacts');
      if (!res.error && res.data) {
        const map: Record<UserId, User> = {};
        for (const user of res.data) {
          map[user.id] = user;
        }
        state.byUserId = map;
      }
    } catch {

    } finally {
      state.isLoading = false;
      state.isInitialized = true;
    }
  }

  function getUserById(id: UserId): User | undefined {
    if (authStore.state.user && authStore.state.user.id == id) {
      return authStore.state.user;
    }
    return state.byUserId[id];
  }

  function getUsername(id: UserId, fallback = 'Unknown User') {
    return getUserById(id)?.username ?? fallback;
  }

  function clearContacts() {
    state.byUserId = {};
    state.isInitialized = false;
    state.isLoading = false;
  }

  return {
    state,
    contactsList,
    fetchContacts,
    getUserById,
    getUsername,
    clearContacts,
  }
}
