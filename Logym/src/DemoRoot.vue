<script setup>
import { provide } from 'vue'
import ToastNotifications from './components/ToastNotifications.vue'
provide('consultarLicencia', () => {}) // El ejecutable Windows permanece intacto.
</script>

<template>
  <div class="min-h-screen bg-[#090a0c] text-white">
    <div class="demo-web-banner" role="note">MODO DEMOSTRACIÓN · Datos ficticios · Sin lector ni licencia</div>
    <router-view />
    <ToastNotifications />
  </div>
</template>

<style scoped>
.demo-web-banner { background:#ee2046;color:white;text-align:center;font-size:11px;font-weight:800;letter-spacing:.09em;padding:8px 12px;position:relative;z-index:24 }
</style>
