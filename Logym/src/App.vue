<script setup>
import { ref, computed, onMounted, onBeforeUnmount, provide } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

import ActivacionLicencia from './components/ActivacionLicencia.vue'
import ToastNotifications from './components/ToastNotifications.vue'
import { mostrarToast } from './services/toast'
import { detalleLectorPublico, mensajeErrorAsistencia } from './services/mensajesLector'
const licenciaValida = ref(false)
const comprobandoLicencia = ref(true)
const consultandoLicencia = ref(false)
const errorLicencia = ref('')
let monitoresIniciados = false
provide('consultarLicencia', () => { consultandoLicencia.value = true })

async function revisarLicencia() {
  comprobandoLicencia.value = true
  errorLicencia.value = ''
  try { licenciaValida.value = await invoke('estado_licencia') === true }
  catch (error) {
    licenciaValida.value = false
    errorLicencia.value = 'No se pudo verificar la licencia. ' + String(error)
  } finally { comprobandoLicencia.value = false }
  if (licenciaValida.value && !monitoresIniciados && !desmontado) {
    monitoresIniciados = true
    await iniciarEventosLector()
  }
}

let desuscribir = null
let desuscribirLector = null
let desuscribirEstadoLector = null
const lector = ref({ estado: 'inicializando', detalle: '', revision: -1,
  usb_presente: false, sdk_inicializado: false, captura_comprobada: false })
const falloConsultaLector = ref(false)
let consultarLectorTimer = null
let avisoLectorPendiente = false
let desmontado = false
const textoLector = computed(() => falloConsultaLector.value ? 'No se pudo consultar el estado del lector' : ({
  inicializando: 'Inicializando lector',
  recuperando: lector.value.sdk_inicializado ? 'Coloca el dedo para comprobar el lector' : 'Reconectando lector',
  no_disponible: 'Lector no disponible',
})[lector.value.estado])

function actualizarLector(payload) {
  if (!['inicializando', 'disponible', 'recuperando', 'no_disponible'].includes(payload?.estado)
    || !Number.isSafeInteger(payload.revision) || payload.revision < lector.value.revision) return
  lector.value = { ...payload, detalle: detalleLectorPublico(payload) }
  falloConsultaLector.value = false
  if (payload.estado === 'no_disponible' && !avisoLectorPendiente) {
    avisoLectorPendiente = true
    mostrarToast({ titulo: 'Lector no disponible', detalle: 'Se reintentará automáticamente. Consulta el indicador del lector si el problema persiste.', tipo: 'error' })
  } else if (payload.estado === 'disponible' && avisoLectorPendiente) {
    avisoLectorPendiente = false
    mostrarToast({ titulo: 'Lectura recuperada', detalle: 'El lector de huellas está listo.', tipo: 'exito' })
  }
}

async function consultarLector() {
  try {
    const estado = await invoke('estado_lector')
    if (!desmontado) actualizarLector(estado)
  } catch (error) {
    if (!desmontado) falloConsultaLector.value = true
    console.error('Consulta del lector:', error)
  }
}
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
  return { titulo: 'No se registró la asistencia', detalle: mensajeErrorAsistencia(detalle), tipo: 'error', simbolo: '×' }
}

async function iniciarEventosLector() {
  try {
    desuscribirEstadoLector = await listen('logym://lector-estado', ({ payload }) => actualizarLector(payload))
  } catch (error) { console.error('Eventos del lector:', error) }
  // Listener primero y snapshot después; revision descarta respuestas atrasadas.
  await consultarLector()
  consultarLectorTimer = setInterval(consultarLector, 2000)
  desuscribirLector = await listen('logym://lector-aviso', ({ payload }) => {
    if (Date.now() - ultimoAvisoLector < 5000) return
    ultimoAvisoLector = Date.now()
    if (payload === 'no_registrado') mostrarToast({ titulo: 'Cliente no registrado', detalle: 'No encontramos una huella registrada que coincida.', tipo: 'aviso' })
    if (payload === 'lectura_invalida') mostrarToast({ titulo: 'Lectura no realizada correctamente', detalle: 'Coloca el dedo completo sobre el sensor e intenta nuevamente.', tipo: 'error' })
    if (payload === 'dedo_rechazado') mostrarToast({ titulo: 'Lectura rechazada por el sensor', detalle: 'El sensor activó su detección de dedo falso. Retira el dedo y vuelve a colocarlo.', tipo: 'aviso' })
  })
  desuscribir = await listen('logym://checkin', ({ payload }) => {
    const { cliente, resultado } = payload || {}
    if (!cliente) return
    mostrarToast({ nombre: cliente.nombre, ...presentarResultado(resultado), duracion: 7500 })
  })
}
onMounted(revisarLicencia)
onBeforeUnmount(() => {
  desmontado = true
  clearInterval(consultarLectorTimer)
  desuscribir?.()
  desuscribirLector?.()
  desuscribirEstadoLector?.()
})
</script>

<template>
  <div class="min-h-screen bg-[#090a0c] text-white">
    <section v-if="comprobandoLicencia" class="app-startup" aria-label="Preparando VIRTUS GYM" aria-busy="true">
      <p class="app-startup__brand">VIRTUS GYM</p>
      <span class="app-startup__spinner" aria-hidden="true"></span>
      <p role="status">Preparando tu acceso…</p>
      <p class="app-startup__hint">La aplicación se está iniciando.</p>
    </section>
    <ActivacionLicencia v-else-if="!licenciaValida || consultandoLicencia"
      :activa="licenciaValida" :error-inicial="errorLicencia"
      @activada="revisarLicencia" @cerrar="consultandoLicencia = false" />
    <div v-if="licenciaValida" v-show="!consultandoLicencia && !comprobandoLicencia"><router-view /></div>
    <div v-if="licenciaValida && !consultandoLicencia && (falloConsultaLector || lector.estado !== 'disponible')" role="status" aria-live="polite" :title="lector.detalle"
      class="fixed bottom-3 left-3 z-[100] flex items-center gap-2 rounded-lg border border-amber-400/20 bg-[#151515] px-3 py-2 text-xs text-amber-200 shadow-md">
      <span aria-hidden="true" class="h-2 w-2 rounded-full bg-amber-400"></span>
      <span>{{ textoLector }}<span v-if="lector.detalle" class="mt-1 block max-w-sm text-[11px] text-amber-200/70">{{ lector.detalle }}</span></span>
    </div>
    <ToastNotifications v-if="licenciaValida" />
  </div>
</template>

