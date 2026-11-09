import { onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri'
import { useRouter } from 'vue-router'
import { mensajeErrorAsistencia } from '../services/mensajesLector'
import { pausarLectorGlobal, buscarClientesCheckin, registrarAsistencia } from '../services/tauriApi'

import { useRenovacion } from './useRenovacion'

/**
 * Vista del lector global. El backend captura y registra asistencias incluso
 * fuera de esta ruta; Vue solamente presenta el resultado y gestiona renovaciones.
 */
export function useCheckIn() {
  const router = useRouter()
  const mensaje = ref('Coloca tu huella para validar tu acceso')
  const estadoCheckin = ref('idle')
  const cargando = ref(false)
  const clienteDetectado = ref(null)
  const modalAbierta = ref(false)
  const lectorActivo = ref(false)
  const estadoLector = ref('inicializando')
  const consultaManual = ref('')
  const resultadosManual = ref([])
  const clienteManual = ref(null)
  const buscandoManual = ref(false)
  const registrandoManual = ref(false)
  const mensajeManual = ref('')
  const busquedaAbierta = ref(false)
  const modalManual = ref(false)
  const asistenciaManualRegistrada = ref(false)
  const lecturasPendientes = ref(0)
  const colaLecturas = []
  let lectorPausadoPorEvento = false
  let regresoLectorTimer = null
  let revisionBusqueda = 0
  watch(consultaManual, () => {
    revisionBusqueda++
    buscandoManual.value = false
    resultadosManual.value = []
    clienteManual.value = null
    mensajeManual.value = ''
  }, { flush: 'sync' })

  async function buscarManual() {
    if (registrandoManual.value || modalAbierta.value) return
    const revision = ++revisionBusqueda
    resultadosManual.value = []
    clienteManual.value = null
    mensajeManual.value = ''
    if (!consultaManual.value.trim()) return
    busquedaAbierta.value = true
    buscandoManual.value = true
    try {
      const clientes = await buscarClientesCheckin(consultaManual.value.trim())
      if (desmontado || revision !== revisionBusqueda) return
      resultadosManual.value = clientes
      mensajeManual.value = clientes.length ? 'Selecciona al cliente y verifica su identidad. Máximo 30 resultados; precisa la búsqueda si es necesario.' : 'No se encontraron clientes activos.'
    } catch (error) {
      console.error('Error al buscar cliente para asistencia manual:', error)
      if (revision === revisionBusqueda) mensajeManual.value = 'No se pudo buscar al cliente. Intenta nuevamente.'
    } finally {
      if (revision === revisionBusqueda) buscandoManual.value = false
    }
  }

  function seleccionarManual(cliente) {
    if (modalAbierta.value || registrandoManual.value || !resultadosManual.value.includes(cliente)) return
    clienteManual.value = cliente
    clienteDetectado.value = cliente
    modalManual.value = true
    modalAbierta.value = true
    asistenciaManualRegistrada.value = false
    estadoCheckin.value = 'idle'
    mensaje.value = 'Verifica la fotografía y los datos del cliente antes de registrar su asistencia.'
  }

  async function confirmarManual() {
    if (!modalManual.value || !clienteManual.value || registrandoManual.value || asistenciaManualRegistrada.value) return
    const cliente = clienteManual.value
    registrandoManual.value = true
    try {
      const resultado = await registrarAsistencia(cliente.id)
      if (desmontado) return
      estadoCheckin.value = 'success'
      mensaje.value = `${resultado}: ${cliente.nombre} (ID #${cliente.id}).`
      asistenciaManualRegistrada.value = true
      regresoLectorTimer = setTimeout(() => {
        cerrarModal().catch(console.error)
      }, 2000)
    } catch (error) {
      console.error('Error al registrar asistencia manual:', error)
      if (!desmontado) {
        estadoCheckin.value = 'error'
        mensaje.value = `No se registró asistencia: ${mensajeErrorAsistencia(error)}`
      }
    } finally {
      registrandoManual.value = false
    }
  }

  function cancelarBusqueda() {
    if (registrandoManual.value) return
    revisionBusqueda++
    consultaManual.value = ''
    resultadosManual.value = []
    clienteManual.value = null
    buscandoManual.value = false
    mensajeManual.value = ''
    busquedaAbierta.value = false
  }
  let dejarDeEscuchar = null
  let dejarDeEscucharEstado = null
  let consultaLectorTimer = null
  let revisionLector = -1
  let desmontado = false

  function actualizarEstadoLector(payload) {
    if (!['inicializando', 'disponible', 'recuperando', 'no_disponible'].includes(payload?.estado)
      || !Number.isSafeInteger(payload.revision) || payload.revision < revisionLector) return
    revisionLector = payload.revision
    estadoLector.value = payload.estado
    lectorActivo.value = payload.estado === 'disponible'
  }

  async function consultarEstadoLector() {
    try {
      const estado = await invoke('estado_lector')
      if (!desmontado) actualizarEstadoLector(estado)
    } catch (error) {
      if (!desmontado) {
        lectorActivo.value = false
        estadoLector.value = 'no_disponible'
        console.error('No se pudo consultar el estado del lector:', error)
      }
    }
  }

  function formatearFecha(valor) {
    if (!valor) return 'Sin fecha'
    const fecha = String(valor).slice(0, 10)
    if (!/^\d{4}-\d{2}-\d{2}$/.test(fecha)) return valor
    const [anio, mes, dia] = fecha.split('-')
    return `${dia}/${mes}/${anio}`
  }
  function etiquetaEstado(estado) {
    return { pendiente: 'Membresía pendiente', vencida: 'Membresía vencida', por_vencer: 'Membresía por vencer', activa: 'Membresía activa' }[estado] || 'Sin fecha de membresía'
  }
  function diasRestantes(cliente) {
    if (cliente.estado_membresia === 'pendiente') return `Comienza el ${formatearFecha(cliente.fecha_inicio_membresia)}`
    if (cliente.estado_membresia === 'vencida') return `Venció hace ${Math.abs(cliente.dias_restantes)} día(s)`
    if (cliente.estado_membresia === 'por_vencer') return `Vence en ${cliente.dias_restantes} día(s)`
    return `${cliente.dias_restantes} día(s) restantes`
  }
  async function recibirLectura(evento) {
    const { cliente, resultado } = evento.payload || {}
    if (!cliente || desmontado) return
    // El backend ya resolvió esta lectura. Conservar cada resultado, incluso
    // si llega durante una confirmación manual o hay otra modal abierta.
    colaLecturas.push({ cliente, resultado })
    lecturasPendientes.value = colaLecturas.length
    if (!modalAbierta.value) mostrarSiguienteLectura()
    if (!lectorPausadoPorEvento) {
      lectorPausadoPorEvento = true
      try { await pausarLectorGlobal(true) }
      catch (error) {
        lectorPausadoPorEvento = false
        console.error('No se pudo pausar la presentación biométrica:', error)
      }
    }
  }
  function mostrarSiguienteLectura() {
    const { cliente, resultado } = colaLecturas.shift()
    lecturasPendientes.value = colaLecturas.length
    modalManual.value = false
    clienteDetectado.value = cliente
    modalAbierta.value = true
    if (resultado === 'registrada') {
      estadoCheckin.value = 'success'
      mensaje.value = `Asistencia registrada: ${cliente.nombre}`
    } else if (resultado === 'duplicada') {
      estadoCheckin.value = 'warning'
      mensaje.value = `${cliente.nombre} ya registró asistencia hoy. No se agregó otra entrada.`
    } else {
      estadoCheckin.value = 'error'
      mensaje.value = resultado === 'membresia_pendiente'
        ? `Membresía pendiente. Comienza el ${formatearFecha(cliente.fecha_inicio_membresia)}.`
        : resultado === 'membresia_vencida'
        ? 'Membresía vencida. Renueva para ingresar.'
        : `No se registró asistencia: ${mensajeErrorAsistencia(resultado)}`
    }
  }
  async function capturarAcceso() {
    // El servicio Rust ya está escuchando; no lanzar una segunda captura.
    mensaje.value = 'El lector permanece activo. Coloca tu huella en el sensor.'
  }
  async function cerrarModal() {
    if (registrandoManual.value) return
    clearTimeout(regresoLectorTimer)
    if (modalManual.value) cancelarBusqueda()
    modalManual.value = false
    asistenciaManualRegistrada.value = false
    modalAbierta.value = false
    clienteDetectado.value = null
    estadoCheckin.value = 'idle'
    mensaje.value = 'Coloca tu huella para validar tu acceso'
    if (colaLecturas.length) {
      mostrarSiguienteLectura()
    } else if (lectorPausadoPorEvento) {
      lectorPausadoPorEvento = false
      await pausarLectorGlobal(false)
    }
  }
  const renovacion = useRenovacion()
  const solicitarRenovacion = () => renovacion.abrir(clienteDetectado.value)
  watch(renovacion.ultimoResultado, resultado => {
    if (resultado && clienteDetectado.value?.id === resultado.id) {
      clienteDetectado.value = resultado
      if (clienteManual.value?.id === resultado.id) clienteManual.value = resultado
    }
  })
  onMounted(async () => {
    try {
      dejarDeEscucharEstado = await listen('logym://lector-estado', ({ payload }) => actualizarEstadoLector(payload))
    } catch (error) { console.error('No se pudo escuchar el estado del lector:', error) }
    await consultarEstadoLector()
    if (!desmontado) consultaLectorTimer = setInterval(consultarEstadoLector, 2000)
    try { dejarDeEscuchar = await listen('logym://checkin', recibirLectura) }
    catch (error) {
      console.error('Error al escuchar el lector:', error)
      mensaje.value = 'No se pudo recibir información del lector. Intenta nuevamente.'
    }
  })
  onBeforeUnmount(() => {
    desmontado = true
    revisionBusqueda++
    clearTimeout(regresoLectorTimer)
    colaLecturas.length = 0
    clearInterval(consultaLectorTimer)
    dejarDeEscucharEstado?.()
    dejarDeEscuchar?.()
    if (lectorPausadoPorEvento) pausarLectorGlobal(false).catch(console.error)
  })
  return {
    consultaManual, resultadosManual, clienteManual, buscandoManual, registrandoManual, mensajeManual,
    busquedaAbierta, modalManual, asistenciaManualRegistrada, lecturasPendientes,
    buscarManual, seleccionarManual, confirmarManual, cancelarBusqueda,
    router, mensaje, estadoCheckin, cargando, clienteDetectado,
    modalAbierta, lectorActivo, estadoLector, formatearFecha, etiquetaEstado,
    diasRestantes, capturarAcceso, cerrarModal, solicitarRenovacion
  }
}
