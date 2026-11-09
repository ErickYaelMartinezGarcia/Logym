import { createRouter, createWebHashHistory } from 'vue-router'

const rutas = [
  { path: '/', name: 'Check-in', component: () => import('./views/CheckIn.vue') },
  { path: '/registro', name: 'Registro de nuevo miembro', component: () => import('./views/Registro.vue') },
  { path: '/dashboard', name: 'Dashboard', component: () => import('./views/Dashboard.vue') },
  { path: '/clientes', name: 'Clientes', component: () => import('./views/AdministracionClientes.vue') },
  { path: '/precios', name: 'Administración de precios', component: () => import('./views/AdministracionPrecios.vue') }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes: rutas
})

export default router
