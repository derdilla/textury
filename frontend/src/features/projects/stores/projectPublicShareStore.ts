import { api } from '@/api';
import { useStorage } from '@vueuse/core'
import { readonly } from 'vue';

export type ProjectPubliclyShared = {
  isShared: boolean,
  shareId: string,
  password: string,
  signUpCount: null | number,
}

export const useProjectPublicShareStore = (projectId: string) => {
  const state = useStorage<ProjectPubliclyShared>(`public-shares-${projectId}`, {
    isShared: false,
    shareId: `${projectId}-shared`,
    password: window.crypto.getRandomValues(new BigUint64Array(1))[0]!.toString(36),
    signUpCount: null
  });

  async function refresh() {
    const share = await api.GET('/project/{id}/publicShare', {
      params: { path: { id: projectId } },
    });
    if (share.response.status === 404 && state.value.isShared) {
      state.value.isShared = false;
      state.value.signUpCount = null;
    } else if (share.data) {
      state.value = {
        isShared: true,
        shareId: share.data.shareId,
        password: share.data.sharePassword,
        signUpCount: share.data.signUpCount,
      }
    }
  }

  function setId(shareId: string): null | string {
    if (state.value.isShared) return 'already shared';
    if (shareId.length < 2) return 'id too short';
    state.value.shareId = shareId;
    return null;
  }

  function setPassword(password: string): null | string {
    if (state.value.isShared) return 'already shared';
    if (password.length < 8) return 'password too short';
    state.value.password = password;
    return null;
  }

  async function share(): Promise<null | string> {
    // Local validation is correct since only changed by methods
    const response = await api.POST('/project/{id}/publicShare', {
      params: { path: { id: projectId } },
      body: {
        shareId: state.value.shareId,
        sharePassword: state.value.password,
      },
    });
    if (!response.data) {
      return 'Request failed'
    }

    if (response.data != 'Success') {
      // TODO: choose localized strings
      return response.data;
    }

    state.value.signUpCount = 0;
    state.value.isShared = true;
    await refresh();
    return null;
  }

  async function unshare() {
    const res = await api.DELETE('/project/{id}/publicShare', {
      params: { path: { id: projectId } },
    });
    if (res.response.status === 200) {
      state.value.isShared = false;
    }
    await refresh();
  }

  return {
    state: readonly(state),
    refresh,
    setId,
    setPassword,
    share,
    unshare,
  }
}
