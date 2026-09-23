import { createRouter, createWebHistory } from 'vue-router'
import RegisterForm from '@/components/login/RegisterForm.vue'
import LoginForm from '@/components/login/LoginForm.vue'
import ProjectsView from '@/views/ProjectsView.vue'
import EditorView from '@/views/EditorView.vue'
import AccountSettingsView from '@/views/AccountSettingsView.vue'
import LoginView from '@/views/LoginView.vue'
import { useAuthStore } from '@/stores/auth'
import { useProjectsStore } from '@/stores/projects'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: LoginView,
      meta: { guestOnly: true },
      children: [
        {
          path: '/register',
          name: 'register',
          component: RegisterForm,
          meta: { guestOnly: true },
        },
        {
          path: '/login',
          name: 'login',
          component: LoginForm,
          meta: { guestOnly: true },
        },
      ]
    },
    {
      path: '/settings/:tabId?/:pageId*',
      name: 'settings',
      component: AccountSettingsView,
      meta: { requiresAuth: true },
    },
    {
      path: '/projects',
      name: 'projects',
      component: ProjectsView,
      meta: { requiresAuth: true },
    },
    {
      path: '/projects/:id',
      name: 'edit',
      component: EditorView,
      meta: { requiresAuth: true },
    },
    {
      path: '/acceptInvite/:token',
      component: ProjectsView,
      meta: { requiresAuth: true },
      beforeEnter: async (to) => {
        const token = to.params.token as string;
        await useProjectsStore().joinProject({ token });
        return { name: 'projects' };
      }
    }
  ],
});

router.beforeEach(async (to) => {
  const authStore = useAuthStore();

  if (!authStore.state.isInitialized) {
    await authStore.checkAuth();
  }

  if (to.meta.requiresAuth && !authStore.isAuthenticated.value) {
    return { name: 'login' };
  }

  if (to.meta.guestOnly && authStore.isAuthenticated.value) {
    return { name: 'projects' };
  }
});

export default router;