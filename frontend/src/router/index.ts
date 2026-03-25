import { createRouter, createWebHistory } from 'vue-router'
import RegisterForm from '@/components/RegisterForm.vue'
import LoginForm from '@/components/LoginForm.vue'
import LoggedOutView from '@/views/LoggedOutView.vue'
import ProjectsView from '@/views/ProjectsView.vue'
import { acceptShare } from '@/services/managementApi'


const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: LoggedOutView,
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

    // TODO: guard these somehow
    {
      path: '/projects',
      name: 'projects',
      component: ProjectsView,
    },
    {
      path: '/projects/:id',
      name: 'edit',
      component: () => import('../views/EditView.vue'),
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
