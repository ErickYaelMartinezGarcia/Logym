import { createRouter, createWebHashHistory } from 'vue-router'
import CheckIn from './views/CheckIn.vue'
import Registro from './views/Registro.vue'
import Dashboard from './views/Dashboard.vue'

const rutas = [
  { path: '/', component: CheckIn },
  { path: '/registro', component: Registro },
  { path: '/dashboard', component: Dashboard }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes: rutas
})

export default router
