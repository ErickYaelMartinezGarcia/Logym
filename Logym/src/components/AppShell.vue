<script setup>
import { ref } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const menuAbierto = ref(false)
const infoAbierta = ref(false)
const version = '0.1.0'

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

      <div style="margin-top:auto;padding:16px 12px 6px">
        <button type="button" @click="infoAbierta = true; cerrarMenu()" style="display:flex;align-items:center;gap:10px;width:100%;padding:12px 14px;border:1px solid #323b46;border-radius:12px;background:#151b22;color:#e7edf3;cursor:pointer;text-align:left">
          <span aria-hidden="true" style="display:grid;place-items:center;width:25px;height:25px;border:1px solid #687786;border-radius:50%;font-weight:800">i</span>
          <span>Información del sistema</span>
        </button>
      </div>
      <div class="drawer-footer">
        <span class="status-dot"></span>
        <span>Sistema operativo</span>
        <small>v0.1</small>
      </div>
    </aside>

    <div v-if="infoAbierta" role="presentation" style="position:fixed;inset:0;background:#000b;z-index:300;display:grid;place-items:center;padding:20px" @click.self="infoAbierta = false">
      <section role="dialog" aria-modal="true" aria-labelledby="info-titulo" style="width:min(430px,100%);background:#161b22;color:#f3f6f9;border:1px solid #36404b;border-radius:18px;padding:25px;box-shadow:0 22px 60px #0009">
        <div style="display:flex;justify-content:space-between;align-items:center;gap:12px">
          <h2 id="info-titulo" style="font-size:21px;font-weight:800">Acerca de Logym</h2>
          <button type="button" aria-label="Cerrar información" @click="infoAbierta = false" style="font-size:26px;color:#d9e1ea;background:none;border:0;cursor:pointer">×</button>
        </div>
        <dl style="display:grid;grid-template-columns:auto 1fr;gap:13px 16px;margin-top:23px;font-size:14px">
          <dt style="color:#aab5c2">Versión</dt><dd>{{ version }}</dd>
          <dt style="color:#aab5c2">Creador</dt><dd>Erick Yael Martinez Garcia</dd>
          <dt style="color:#aab5c2">Soporte</dt><dd>Correo de soporte: pendiente de configurar</dd>
          <dt style="color:#aab5c2">Nombre</dt><dd><strong>Virtus Gym Sistem</strong></dd>
        </dl>
      </section>
    </div>

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
