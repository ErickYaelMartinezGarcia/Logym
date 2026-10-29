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
    try {
      capturaEnCurso.value = true
      cargando.value = true
      const template = await capturarHuella()
      lectorActivo.value = true
      if (!forzar && template === ultimoTemplate.value) return
      ultimoTemplate.value = template
      const cliente = await buscarClientePorHuella(template)
      if (!cliente) {
        clienteDetectado.value = null
        mensaje.value = 'Huella leída, pero no coincide con un miembro registrado.'
        return
      }
      clienteDetectado.value = cliente
      modalAbierta.value = true
      if (cliente.estado_membresia === 'vencida') {
        mensaje.value = 'Membresía vencida detectada. Se recomienda autorizar una renovación.'
      } else if (cliente.estado_membresia === 'sin_fecha') {
        mensaje.value = 'El cliente no tiene una membresía válida configurada.'
      } else {
        const registrada = await registrarAsistencia(cliente.id)
        mensaje.value = registrada
          ? `Acceso válido registrado: ${cliente.nombre}`
          : `Acceso válido: ${cliente.nombre}. La asistencia de hoy ya estaba registrada.`
        if (cliente.estado_membresia === 'por_vencer') mensaje.value += ' Se recomienda renovar pronto.'
      }
    } catch (error) {
      console.error('Error en check-in:', error)
      mensaje.value = 'No se pudo leer el dispositivo biométrico.'
    } finally {
      cargando.value = false
      capturaEnCurso.value = false
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
  })

  return {
    router, mensaje, cargando, procesandoRenovacion, clienteDetectado, modalAbierta,
    confirmandoRenovacion, lectorActivo, lecturaAutomatica, formatearFecha, etiquetaEstado,
    diasRestantes, capturarAcceso, cerrarModal, solicitarRenovacion, renovar
  }
}
