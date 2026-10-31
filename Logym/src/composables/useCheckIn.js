import { onMounted, onBeforeUnmount, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useRouter } from 'vue-router'
import { pausarLectorGlobal, renovarMembresia } from '../services/tauriApi'

/**
 * Vista del lector global. El backend captura y registra asistencias incluso
 * fuera de esta ruta; Vue solamente presenta el resultado y gestiona renovaciones.
 */
export function useCheckIn() {
  const router = useRouter()
  const mensaje = ref('Coloca tu huella para validar tu acceso')
  const estadoCheckin = ref('idle')
  const cargando = ref(false)
  const procesandoRenovacion = ref(false)
  const clienteDetectado = ref(null)
  const modalAbierta = ref(false)
  const confirmandoRenovacion = ref(false)
  const lectorActivo = ref(true)
  let dejarDeEscuchar = null

  function formatearFecha(valor) {
    if (!valor) return 'Sin fecha'
    const fecha = String(valor).slice(0, 10)
    if (!/^\d{4}-\d{2}-\d{2}$/.test(fecha)) return valor
    const [anio, mes, dia] = fecha.split('-')
    return `${dia}/${mes}/${anio}`
  }
  function etiquetaEstado(estado) {
    return { vencida: 'Membresía vencida', por_vencer: 'Membresía por vencer', activa: 'Membresía activa' }[estado] || 'Sin fecha de membresía'
  }
  function diasRestantes(cliente) {
    if (cliente.estado_membresia === 'vencida') return `Venció hace ${Math.abs(cliente.dias_restantes)} día(s)`
    if (cliente.estado_membresia === 'por_vencer') return `Vence en ${cliente.dias_restantes} día(s)`
    return `${cliente.dias_restantes} día(s) restantes`
  }
  async function recibirLectura(evento) {
    const { cliente, resultado } = evento.payload || {}
    if (!cliente) return
    clienteDetectado.value = cliente
    modalAbierta.value = true
    await pausarLectorGlobal(true)
    if (resultado === 'registrada') {
      estadoCheckin.value = 'success'
      mensaje.value = `Asistencia registrada: ${cliente.nombre}`
    } else if (resultado === 'duplicada') {
      estadoCheckin.value = 'success'
      mensaje.value = `Ya registró asistencia hoy: ${cliente.nombre}`
    } else {
      estadoCheckin.value = 'error'
      mensaje.value = resultado === 'membresia_vencida' ? 'Membresía vencida. Renueva para ingresar.' : `No se registró asistencia: ${resultado}`
    }
  }
  async function capturarAcceso() {
    // El servicio Rust ya está escuchando; no lanzar una segunda captura.
    mensaje.value = 'El lector permanece activo. Coloca tu huella en el sensor.'
  }
  async function cerrarModal() {
    modalAbierta.value = false
    confirmandoRenovacion.value = false
    clienteDetectado.value = null
    estadoCheckin.value = 'idle'
    mensaje.value = 'Coloca tu huella para validar tu acceso'
    await pausarLectorGlobal(false)
  }
  function solicitarRenovacion() { confirmandoRenovacion.value = true }
  async function renovar() {
    if (!clienteDetectado.value || procesandoRenovacion.value) return
    procesandoRenovacion.value = true
    try {
      clienteDetectado.value = await renovarMembresia(clienteDetectado.value.id)
      estadoCheckin.value = 'success'
      confirmandoRenovacion.value = false
      mensaje.value = `Membresía renovada para ${clienteDetectado.value.nombre}. Vigente hasta ${formatearFecha(clienteDetectado.value.fecha_fin_membresia)}.`
    } catch (error) {
      estadoCheckin.value = 'error'
      mensaje.value = `No se pudo renovar: ${String(error)}`
    } finally {
      procesandoRenovacion.value = false
    }
  }
  onMounted(async () => {
    try { dejarDeEscuchar = await listen('logym://checkin', recibirLectura) }
    catch (error) { mensaje.value = `No se pudo escuchar el lector: ${String(error)}` }
  })
  onBeforeUnmount(() => {
    dejarDeEscuchar?.()
    if (modalAbierta.value) pausarLectorGlobal(false).catch(console.error)
  })
  return {
    router, mensaje, estadoCheckin, cargando, procesandoRenovacion, clienteDetectado,
    modalAbierta, confirmandoRenovacion, lectorActivo, formatearFecha, etiquetaEstado,
    diasRestantes, capturarAcceso, cerrarModal, solicitarRenovacion, renovar
  }
}
