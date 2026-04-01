import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import Tavern from '../views/Tavern.vue'
import Versions from '../views/Versions.vue'
import Extensions from '../views/Extensions.vue'
import Tools from '../views/Tools.vue'
import Resources from '../views/Resources.vue'
import Console from '../views/Console.vue'
import Settings from '../views/Settings.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home,
  },
  {
    path: '/tavern',
    name: 'Tavern',
    component: Tavern,
  },
  {
    path: '/versions',
    name: 'Versions',
    component: Versions,
  },
  {
    path: '/extensions',
    name: 'Extensions',
    component: Extensions,
  },
  {
    path: '/tools',
    name: 'Tools',
    component: Tools,
  },
  {
    path: '/resources',
    name: 'Resources',
    component: Resources,
  },
  {
    path: '/console',
    name: 'Console',
    component: Console,
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Settings,
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
