<script setup>
import { ref } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const menuAbierto = ref(false)

const enlaces = [
   { to: '/dashboard', label: 'Dashboard', caption: 'Actividad del gimnasio', icon: 'grid' },
  { to: '/', label: 'Check-in', caption: 'Acceso de miembros', icon: 'fingerprint' },
  { to: '/registro', label: 'Nuevo miembro', caption: 'Alta y biometría', icon: 'user-plus' },
  { to: '/clientes', label: 'Administrar clientes', caption: 'Editar y controlar accesos', icon: 'users' }
]

function cerrarMenu() {
  menuAbierto.value = false
}
</script>

<template>
  <div class="app-shell">
    <div v-if="menuAbierto" class="drawer-backdrop" @click="cerrarMenu"></div>

    <aside class="app-drawer" :class="{ 'app-drawer--open': menuAbierto }">
      <div class="drawer-brand">
        <div class="brand-mark">V</div>
        <div>
          <p class="brand-name">VIRTUS</p>
          <p class="brand-subtitle">GYM / CONTROL</p>
        </div>
        <button class="drawer-close" aria-label="Cerrar menú" @click="cerrarMenu">×</button>
      </div>

      <div class="drawer-rule"></div>
      <p class="drawer-section-label">NAVEGACIÓN</p>
      <nav class="drawer-nav" aria-label="Navegación principal">
        <router-link
          v-for="enlace in enlaces"
          :key="enlace.to"
          :to="enlace.to"
          class="drawer-link"
          :class="{ 'drawer-link--active': route.path === enlace.to }"
          @click="cerrarMenu"
        >
          <span class="drawer-icon" aria-hidden="true">
            <svg v-if="enlace.icon === 'fingerprint'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><path d="M12 11a3 3 0 0 1 3 3v1"/><path d="M8.5 14a3.5 3.5 0 0 1 7 0v3"/><path d="M5.5 14a6.5 6.5 0 0 1 13 0v1"/><path d="M12 5a9 9 0 0 1 9 9"/><path d="M3 14a9 9 0 0 1 9-9"/><path d="M8 20a6 6 0 0 0 8 0"/></svg>
            <svg v-else-if="enlace.icon === 'user-plus'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><circle cx="9" cy="8" r="3"/><path d="M3 20a6 6 0 0 1 12 0M18 8v6M15 11h6"/></svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7"><rect x="4" y="4" width="6" height="6" rx="1"/><rect x="14" y="4" width="6" height="6" rx="1"/><rect x="4" y="14" width="6" height="6" rx="1"/><rect x="14" y="14" width="6" height="6" rx="1"/></svg>
          </span>
          <span><strong>{{ enlace.label }}</strong><small>{{ enlace.caption }}</small></span>
        </router-link>
      </nav>

      <div class="drawer-footer">
        <span class="status-dot"></span>
        <span>Sistema operativo</span>
        <small>v0.1</small>
      </div>
    </aside>

    <div class="app-content">
      <header class="topbar">
        <button class="menu-button" aria-label="Abrir menú" @click="menuAbierto = true">
          <span></span><span></span><span></span>
        </button>
        <div class="topbar-context">
          <span class="topbar-kicker">VIRTUS GYM</span>
          <span class="topbar-divider"></span>
          <span class="topbar-page">{{ route.name || 'Control central' }}</span>
        </div>
        <div class="topbar-date">{{ new Date().toLocaleDateString('es-MX', { weekday: 'short', day: 'numeric', month: 'short' }) }}</div>
      </header>
      <main class="page-content"><slot /></main>
    </div>
  </div>
</template>
