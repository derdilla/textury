import { createRouter, createWebHistory } from 'vue-router'
import RegisterForm from '@/components/login/RegisterForm.vue'
import LoginForm from '@/components/login/LoginForm.vue'
import ProjectsView from '@/views/ProjectsView.vue'
import EditView from '@/views/EditView.vue'
import { acceptShare } from '@/services/api/managementApi'
import AccountSettingsView from '@/views/AccountSettingsView.vue'
import LoginView from '@/views/LoginView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: LoginView,
      children: [
        {
          path: '/register',
          name: 'register',
          component: RegisterForm,
        },
        {
          path: '/login',
          name: 'login',
          component: LoginForm,
        },
      ]
    },
    {
      path: '/settings/:tabId?/:pageId*',
      name: 'settings',
      component: AccountSettingsView,
    },
    // TODO: guard these somehow
    {
      path: '/projects',
      name: 'projects',
      component: ProjectsView,
    },
    {
      path: '/projects/:id',
      name: 'edit',
      component: EditView,
    },
    {
      path: '/acceptInvite/:token',
      component: ProjectsView,
      beforeEnter: async (to, _) => {
        const token = to.params.token as string;
        acceptShare(token);
      }
    }
  ],
})

export default router