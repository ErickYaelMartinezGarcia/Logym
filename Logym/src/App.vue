<script setup>
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'

const aviso = ref(null)
let desuscribir = null
let ocultar = null

// Un solo aviso global: el lector sigue activo aunque el usuario navegue.
onMounted(async () => {
  desuscribir = await listen('logym://checkin', ({ payload }) => {
    const { cliente, resultado } = payload || {}
    if (!cliente) return
    aviso.value = {
      nombre: cliente.nombre,
      texto: resultado === 'registrada' ? 'Asistencia registrada'
        : resultado === 'duplicada' ? 'Asistencia registrada anteriormente hoy'
        : resultado === 'membresia_vencida' ? 'Membresía vencida'
        : 'No se pudo registrar la asistencia',
      correcto: resultado === 'registrada' || resultado === 'duplicada'
    }
    if (ocultar) clearTimeout(ocultar)
    ocultar = setTimeout(() => { aviso.value = null }, 6500)
  })
})
onBeforeUnmount(() => {
  desuscribir?.()
  if (ocultar) clearTimeout(ocultar)
})
</script>

<template>
  <div class="min-h-screen bg-[#090a0c] text-white">
    <router-view />
    <div v-if="aviso" class="aviso-global" :class="{ 'aviso-global--ok': aviso.correcto }" role="status">
      <strong>{{ aviso.texto }}</strong>
      <span>{{ aviso.nombre }}</span>
      <button aria-label="Cerrar aviso" @click="aviso = null">×</button>
    </div>
  </div>
</template>

<style scoped>
.aviso-global{position:fixed;right:22px;bottom:22px;z-index:110;display:flex;align-items:center;gap:14px;max-width:min(430px,calc(100vw - 44px));padding:17px 20px;border:1px solid #c3485d;border-radius:13px;background:#311c25;box-shadow:0 12px 40px #0009;color:#fff}
.aviso-global--ok{border-color:#39946a;background:#153d2c}
.aviso-global strong,.aviso-global span{font-size:13px}
.aviso-global button{border:0;background:transparent;color:#fff;font-size:22px;cursor:pointer}
</style>
