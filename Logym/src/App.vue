<script setup>
import { onMounted, onBeforeUnmount } from 'vue'
import { listen } from '@tauri-apps/api/event'

import ToastNotifications from './components/ToastNotifications.vue'
import { mostrarToast } from './services/toast'
let desuscribir = null
let desuscribirLector = null
let ultimoAvisoLector = 0

// La respuesta de Rust conserva la causa exacta del rechazo; no ocultar errores.
function presentarResultado(resultado) {
  if (resultado === 'registrada') return { titulo: 'Acceso autorizado', detalle: 'Asistencia registrada correctamente.', tipo: 'exito', simbolo: '✓' }
  if (resultado === 'duplicada') return { titulo: 'Asistencia ya registrada', detalle: 'Este cliente ya registró su entrada hoy. No se duplicó la asistencia.', tipo: 'aviso', simbolo: '!' }
  if (resultado === 'membresia_pendiente') return { titulo: 'Membresía pendiente', detalle: 'La fecha de inicio aún no llega. Consulta la fecha de inicio del cliente.', tipo: 'aviso', simbolo: '!' }
  if (resultado === 'membresia_vencida') return { titulo: 'Membresía vencida', detalle: 'Renueva la membresía para permitir el acceso.', tipo: 'error', simbolo: '×' }
  const detalle = String(resultado || 'Error desconocido').replace(/^error:\s*/i, '').trim()
  if (detalle.includes('membresía no vigente')) return { titulo: 'Membresía no vigente', detalle: 'La fecha de inicio todavía no llega o la membresía ya terminó.', tipo: 'error', simbolo: '×' }
  if (detalle.includes('desactivado')) return { titulo: 'Cliente desactivado', detalle: 'Reactiva el cliente desde Administración para permitir su acceso.', tipo: 'error', simbolo: '×' }
  return { titulo: 'No se registró la asistencia', detalle, tipo: 'error', simbolo: '×' }
}

onMounted(async () => {
  desuscribirLector = await listen('logym://lector-aviso', ({ payload }) => {
    if (Date.now() - ultimoAvisoLector < 5000) return
    ultimoAvisoLector = Date.now()
    if (payload === 'no_registrado') mostrarToast({ titulo: 'Cliente no registrado', detalle: 'No encontramos una huella registrada que coincida.', tipo: 'aviso' })
    if (payload === 'lectura_invalida') mostrarToast({ titulo: 'Lectura no realizada correctamente', detalle: 'Coloca el dedo completo sobre el sensor e intenta nuevamente.', tipo: 'error' })
  })
  desuscribir = await listen('logym://checkin', ({ payload }) => {
    const { cliente, resultado } = payload || {}
    if (!cliente) return
    mostrarToast({ nombre: cliente.nombre, ...presentarResultado(resultado), duracion: 7500 })
  })
})
onBeforeUnmount(() => {
  desuscribir?.()
  desuscribirLector?.()
})
</script>

<template>
  <div class="min-h-screen bg-[#090a0c] text-white">
    <router-view />
    <ToastNotifications />
  </div>
</template>

