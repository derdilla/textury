import 'beercss';
import { useAuthStore } from '@/shared/stores/authStore';
import { useStorage } from '@vueuse/core';
import { watch } from 'vue';

export function setupTheme() {
  // This has nothing to do with our brand colors, but it results in a fitting default that matches our color direction.
  ui("theme", "#ff37aa");

  // Automatic switching between light and dark mode without page reload.
  const modeQuery = window.matchMedia("(prefers-color-scheme: dark)");
  const savedTheme = useStorage<string>('last-known-color-scheme', 'auto');
  const authStore = useAuthStore();

  const updateTheme = () => {
    let scheme = 'auto';

    if (authStore.isAuthenticated && authStore.state.user) {
      const userSettings = authStore.state.user.settings as Record<string, unknown> | undefined;
      const rawScheme = userSettings?.colorScheme;

      if (typeof rawScheme === 'string' && rawScheme.length > 0) {
        scheme = rawScheme;
        savedTheme.value = rawScheme;
      } else {
        scheme = typeof savedTheme.value === 'string' ? savedTheme.value : 'auto';
      }
    } else {
      scheme = typeof savedTheme.value === 'string' ? savedTheme.value : 'auto';
    }

    const themeMeta = document.getElementById('theme-color-meta');

    if (scheme === 'dark') {
      ui("mode", "dark");
      if (themeMeta) themeMeta.setAttribute('content', '#171214');
    } else if (scheme === 'light') {
      ui("mode", "light");
      if (themeMeta) themeMeta.setAttribute('content', '#f7ebed');
    } else {
      const isDark = modeQuery.matches;
      ui("mode", isDark ? "dark" : "light");
      if (themeMeta) themeMeta.setAttribute('content', isDark ? '#171214' : '#f7ebed');
    }
  };

  modeQuery.addEventListener("change", updateTheme);

  watch(
    () => {
      const userSettings = authStore.state.user?.settings as Record<string, unknown> | undefined;
      return [authStore.isAuthenticated, userSettings?.colorScheme];
    },
    () => {
      updateTheme();
    },
    { immediate: true }
  );
}
