import { createRouter, createWebHashHistory } from 'vue-router'
import CheckIn from './views/CheckIn.vue'
import Registro from './views/Registro.vue'
import Dashboard from './views/Dashboard.vue'
import AdministracionClientes from './views/AdministracionClientes.vue'

const rutas = [
  { path: '/', name: 'Check-in', component: CheckIn },
  { path: '/registro', name: 'Nuevo miembro', component: Registro },
  { path: '/dashboard', name: 'Dashboard', component: Dashboard },
  { path: '/clientes', name: 'Clientes', component: AdministracionClientes }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes: rutas
})

export default router
