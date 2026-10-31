import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import {
  buscarClientePorHuella,
  capturarHuella,
  inicializarBaseDeDatos,
  registrarAsistencia,
  renovarMembresia
} from '../services/tauriApi'

export function useCheckIn() {
  const router = useRouter()
  const mensaje = ref('Coloca tu huella para validar tu acceso')
  const estadoCheckin = ref('idle')
  const cargando = ref(false)
  const procesandoRenovacion = ref(false)
  const clienteDetectado = ref(null)
  const modalAbierta = ref(false)
  const confirmandoRenovacion = ref(false)
  const lectorActivo = ref(false)
  const lecturaAutomatica = ref(true)
  const ultimoTemplate = ref('')
  const capturaEnCurso = ref(false)
  let intervaloLector = null
  let reinicioEstado = null
  let ultimoIntentoFallido = 0

  function formatearFecha(valor) {
    if (!valor) return 'Sin fecha'
    const fecha = String(valor).slice(0, 10)
    if (!/^\d{4}-\d{2}-\d{2}$/.test(fecha)) return valor
    const [anio, mes, dia] = fecha.split('-')
    return `${dia}/${mes}/${anio}`
  }

  function etiquetaEstado(estado) {
    return { vencida: 'Membresía vencida', por_vencer: 'Membresía por vencer', activa: 'Membresía activa' }[estado]
      || 'Sin fecha de membresía'
  }

  function diasRestantes(cliente) {
    if (cliente.estado_membresia === 'vencida') return `Venció hace ${Math.abs(cliente.dias_restantes)} día(s)`
    if (cliente.estado_membresia === 'por_vencer') return `Vence en ${cliente.dias_restantes} día(s)`
    if (cliente.estado_membresia === 'activa') return `${cliente.dias_restantes} día(s) restantes`
    return 'Configura fechas de membresía'
  }

async function capturarAcceso(forzar = false) {
  if (capturaEnCurso.value) return
  // Anti-spam: si falló hace menos de 2s, no lo intentes de nuevo
  if (!forzar && Date.now() - ultimoIntentoFallido < 2000) return

  try {
    capturaEnCurso.value = true
    cargando.value = true
    estadoCheckin.value = 'processing'
    mensaje.value = 'Leyendo huella...'

    const template = await capturarHuella()
    lectorActivo.value = true

    // Si es la misma huella que ya leíste hace <3s, ignorala
    if (!forzar && template === ultimoTemplate.value) {
      mensaje.value = 'Quita el dedo del lector...'
      // Forzamos 2.5s de espera para que quite el dedo
      await new Promise(r => setTimeout(r, 2500))
      return
    }
    ultimoTemplate.value = template

    mensaje.value = 'Buscando en la base local...'
    const cliente = await buscarClientePorHuella(template)

    if (!cliente) {
      clienteDetectado.value = null
      estadoCheckin.value = 'error'
      mensaje.value = 'Huella no registrada. Intenta de nuevo.'
      ultimoIntentoFallido = Date.now()
      // Delay largo para que quite el dedo
      await new Promise(r => setTimeout(r, 2000))
      return
    }

    clienteDetectado.value = cliente
    modalAbierta.value = true

    if (cliente.estado_membresia === 'vencida') {
      estadoCheckin.value = 'error'
      mensaje.value = 'Membresía vencida. Renovar.'
    } else if (cliente.estado_membresia === 'sin_fecha') {
      estadoCheckin.value = 'error'
      mensaje.value = 'Sin membresía válida.'
    } else {
      const registrada = await registrarAsistencia(cliente.id)
      mensaje.value = registrada
        ? `Acceso: ${cliente.nombre}`
        : `Ya registrado hoy: ${cliente.nombre}`
      estadoCheckin.value = 'success'
      // Si fue éxito, dale 4 segundos para que se vaya
      await new Promise(r => setTimeout(r, 4000))
      ultimoTemplate.value = null // Resetea para que pueda volver a entrar
    }

  } catch (error) {
    console.error('Error en check-in:', error)
    estadoCheckin.value = 'error'
    mensaje.value = 'No se pudo leer el lector. Revisa el USB.'
    ultimoIntentoFallido = Date.now()
    await new Promise(r => setTimeout(r, 2000))
  } finally {
    cargando.value = false
    capturaEnCurso.value = false
    if (reinicioEstado) window.clearTimeout(reinicioEstado)
    reinicioEstado = window.setTimeout(() => {
      // Solo resetea si no hay un cliente en modal
      if (!modalAbierta.value) {
        estadoCheckin.value = 'idle'
        mensaje.value = 'Coloca tu huella para validar tu acceso'
      }
    }, 1500)
  }
}

  function iniciarLecturaAutomatica() {
    intervaloLector = window.setInterval(() => capturarAcceso(), 1400)
    capturarAcceso()
  }

  function cerrarModal() {
    modalAbierta.value = false
    confirmandoRenovacion.value = false
  }

  function solicitarRenovacion() {
    confirmandoRenovacion.value = true
  }

  async function renovar() {
    if (!clienteDetectado.value) return
    try {
      procesandoRenovacion.value = true
      clienteDetectado.value = await renovarMembresia(clienteDetectado.value.id)
      confirmandoRenovacion.value = false
      mensaje.value = `Membresía renovada para ${clienteDetectado.value.nombre}. Nuevo inicio: ${formatearFecha(clienteDetectado.value.fecha_inicio_membresia)}.`
    } catch (error) {
      console.error('Error al renovar:', error)
      mensaje.value = 'No se pudo renovar la membresía. Intenta de nuevo.'
    } finally {
      procesandoRenovacion.value = false
    }
  }

  onMounted(async () => {
    try {
      await inicializarBaseDeDatos()
      iniciarLecturaAutomatica()
    } catch (error) {
      console.error('Error al iniciar el lector:', error)
      mensaje.value = 'No se pudo abrir la base de datos local.'
    }
  })

  onBeforeUnmount(() => {
    if (intervaloLector) window.clearInterval(intervaloLector)
    if (reinicioEstado) window.clearTimeout(reinicioEstado)
  })

  return {
    router, mensaje, estadoCheckin, cargando, procesandoRenovacion, clienteDetectado, modalAbierta,
    confirmandoRenovacion, lectorActivo, lecturaAutomatica, formatearFecha, etiquetaEstado,
    diasRestantes, capturarAcceso, cerrarModal, solicitarRenovacion, renovar
  }
}
