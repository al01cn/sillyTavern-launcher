import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import Tavern from '../views/Tavern.vue'
import Versions from '../views/Versions.vue'
import Plugins from '../views/Plugins.vue'
import Tools from '../views/Tools.vue'
import Console from '../views/Console.vue'
import Settings from '../views/Settings.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home
  },
  {
    path: '/tavern',
    name: 'Tavern',
    component: Tavern
  },
  {
    path: '/versions',
    name: 'Versions',
    component: Versions
  },
  {
    path: '/plugins',
    name: 'Plugins',
    component: Plugins
  },
  {
    path: '/tools',
    name: 'Tools',
    component: Tools
  },
  {
    path: '/console',
    name: 'Console',
    component: Console
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Settings
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
