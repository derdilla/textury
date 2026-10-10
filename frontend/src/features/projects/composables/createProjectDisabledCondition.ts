import { useAccountSettingsStore } from "@/features/settings/stores/accountSettingsStore";
import { useAuthStore } from "@/shared/stores/authStore";
import { computed } from "vue";

export function useCreateProjectsCondition() {
  const authStore = useAuthStore();
  const accountSettingsStore = useAccountSettingsStore();

  const isCreateDisabled = computed<boolean>(() => createDisabledReason.value !== null);
  const createDisabledReason = computed<string | null>(() => {
    if (!authStore.state.user) return "No user loaded";
    if (authStore.state.user.tiedToProject) return "You are tied to a project";
    if (!accountSettingsStore.state.serverSettings?.userProjects
      && !authStore.state.user.isAdmin) return "Only administrators are allowed to create projects on this instance";
    return null;
  });

  return { isCreateDisabled, createDisabledReason };
}
