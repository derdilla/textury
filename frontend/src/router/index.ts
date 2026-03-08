import { createRouter, createWebHistory } from 'vue-router'
import RegisterForm from '../components/RegisterForm.vue'
import LoginForm from '../components/LoginForm.vue'
import LoggedOutView from '../views/LoggedOutView.vue'
import ProjectsView from '../views/ProjectsView.vue'

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
  ],
})

export default router
