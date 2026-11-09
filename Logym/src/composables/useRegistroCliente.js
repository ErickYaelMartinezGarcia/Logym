import { onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { capturarHuella, compararHuella, guardarCliente, inicializarBaseDeDatos, pausarLectorGlobal, listarTiposMembresia, obtenerPrecioInscripcion } from '../services/tauriApi'
import { useCamera } from './useCamera'
import { mensajeErrorLector, mensajeErrorAlta } from '../services/mensajesLector'

export function useRegistroCliente() {
  const tiposMembresia = ref([])
  const tipoMembresiaId = ref('')
  const incluirInscripcion = ref(true)
  const precioInscripcion = ref(0)
  const nombre = ref('')
  const telefono = ref('')
  const telefonoEmergencia = ref('')
  const fechaRegistro = ref('')
  const fechaInicioMembresia = ref('')
  const fechaFinMembresia = ref('')
  const sinHuella = ref(false)
  const template1 = ref('')
  const template2 = ref('')
  const capturandoHuella = ref(false)
  const guardando = ref(false)
  const mensaje = ref('Completa los datos y captura la biometría')
  const tipoMensaje = ref('neutral')

  function mostrarMensaje(texto, tipo = 'neutral') {
    mensaje.value = texto
    tipoMensaje.value = tipo
  }

  function fechaLocal(date = new Date()) {
    const anio = date.getFullYear()
    const mes = String(date.getMonth() + 1).padStart(2, '0')
    const dia = String(date.getDate()).padStart(2, '0')
    return `${anio}-${mes}-${dia}`
  }

  function sumarDias(fechaISO, dias) {
    const fecha = new Date(`${fechaISO}T00:00:00`)
    fecha.setDate(fecha.getDate() + dias)
    return fechaLocal(fecha)
  }

  watch([tipoMembresiaId, fechaInicioMembresia, tiposMembresia], () => {
    const plan = tiposMembresia.value.find(p => String(p.id) === tipoMembresiaId.value)
    if (!plan || !fechaInicioMembresia.value) return
    const inicio = new Date(`${fechaInicioMembresia.value}T12:00:00`)
    if (Number.isNaN(inicio.getTime())) return
    const diaOriginal = inicio.getDate()
    inicio.setDate(1)
    inicio.setMonth(inicio.getMonth() + plan.duracion_meses)
    const ultimoDia = new Date(inicio.getFullYear(), inicio.getMonth() + 1, 0).getDate()
    inicio.setDate(Math.min(diaOriginal, ultimoDia))
    fechaFinMembresia.value = fechaLocal(inicio)
  })

  watch(sinHuella, () => {
    template1.value = ''
    template2.value = ''
    mostrarMensaje(sinHuella.value ? 'Registro sin huella: el acceso se confirmará manualmente.' : 'Captura las dos huellas para continuar.')
  }, { flush: 'sync' })

  const camera = useCamera(mostrarMensaje)

  async function capturarHuellaCliente(numero) {
    if (sinHuella.value || capturandoHuella.value || guardando.value) return
    if (numero === 2 && !template1.value) {
      mostrarMensaje('Primero captura correctamente la primera huella.', 'error')
      return
    }
    capturandoHuella.value = true
    mostrarMensaje(`Esperando la huella ${numero}. Coloca el dedo completamente sobre el lector.`)
    try {
      const template = await capturarHuella()
      // El SDK puede devolver null cuando apenas se roza el sensor.
      // Nunca anunciar éxito ni marcar el botón sin una plantilla real.
      if (typeof template !== 'string' || !/^[a-fA-F0-9]{800}$/.test(template)) {
        mostrarMensaje('No se obtuvo una huella válida. Coloca el dedo completo y vuelve a intentar.', 'error')
        return
      }
      if (numero === 1) {
        template1.value = template
        template2.value = '' // Una nueva primera huella invalida la confirmación anterior.
        mostrarMensaje('Primera huella capturada. Retira el dedo y vuelve a colocarlo para confirmar.', 'success')
      } else {
        const coincide = await compararHuella(template1.value, template)
        if (!coincide) {
          template2.value = ''
          mostrarMensaje('La segunda huella no coincide con la primera. Retira el dedo e intenta nuevamente.', 'error')
          return
        }
        template2.value = template
        mostrarMensaje('Segunda huella confirmada. Ya puedes guardar al miembro.', 'success')
      }
    } catch (error) {
      console.error(`Error al capturar huella ${numero}:`, error)
      mostrarMensaje(mensajeErrorLector(error), 'error')
    } finally {
      capturandoHuella.value = false
    }
  }

  async function guardar() {
    if (guardando.value) return
    if (capturandoHuella.value) {
      mostrarMensaje('Espera a que termine la captura de huella.', 'error')
      return
    }
    if (!nombre.value.trim() || (!sinHuella.value && (!template1.value || !template2.value))) {
      mostrarMensaje(sinHuella.value ? 'Completa el nombre del miembro.' : 'Completa el nombre y las dos capturas de huella.', 'error')
      return
    }
    if (!fechaRegistro.value || !fechaInicioMembresia.value || !fechaFinMembresia.value) {
      mostrarMensaje('Captura las fechas de registro, inicio y fin de membresía.', 'error')
      return
    }
    if (fechaFinMembresia.value < fechaInicioMembresia.value) {
      mostrarMensaje('La fecha de fin no puede ser anterior al inicio de membresía.', 'error')
      return
    }
    const telefonoValido = (valor) => !valor.trim() || /^[0-9]{10}$/.test(valor.trim())
    if (nombre.value.trim().length > 120 || [...nombre.value].some(ch => ch.charCodeAt(0) < 32 || ch.charCodeAt(0) === 127)) {
      mostrarMensaje('El nombre debe tener máximo 120 caracteres y no contener caracteres de control.', 'error')
      return
    }
    if (!telefonoValido(telefono.value) || !telefonoValido(telefonoEmergencia.value)) {
      mostrarMensaje('Los teléfonos deben contener exactamente 10 dígitos numéricos (o quedar vacíos).', 'error')
      return
    }
    if (!tipoMembresiaId.value) { mostrarMensaje('Selecciona una membresía.', 'error'); return }
    guardando.value = true
    try {
      const huellasCoinciden = sinHuella.value || await compararHuella(template1.value, template2.value)
      if (!huellasCoinciden) {
        mostrarMensaje('Las capturas no coinciden. Repite el registro biométrico.', 'error')
        return
      }
      await guardarCliente({
        nombre: nombre.value.trim(),
        tipoMembresiaId: Number(tipoMembresiaId.value),
        incluirInscripcion: incluirInscripcion.value,
        telefono: telefono.value.trim(),
        telefonoEmergencia: telefonoEmergencia.value.trim(),
        fechaRegistro: fechaRegistro.value,
        fechaInicioMembresia: fechaInicioMembresia.value,
        fechaFinMembresia: fechaFinMembresia.value,
        fotoBase64: camera.fotoBase64.value,
        sinHuella: sinHuella.value,
        templateHuella: sinHuella.value ? '' : template1.value
      })
      nombre.value = ''
      telefono.value = ''
      telefonoEmergencia.value = ''
      template1.value = ''
      template2.value = ''
      sinHuella.value = false
      camera.fotoBase64.value = ''
      fechaRegistro.value = fechaLocal()
      fechaInicioMembresia.value = fechaRegistro.value
      fechaFinMembresia.value = sumarDias(fechaInicioMembresia.value, 30)
      mostrarMensaje('Miembro registrado. Ya puede utilizar el check-in.', 'success')
    } catch (error) {
      console.error('Error al guardar miembro:', error)
      mostrarMensaje(mensajeErrorAlta(error), 'error')
    } finally {
      guardando.value = false
    }
  }

  onMounted(async () => {
    await pausarLectorGlobal(true)
    const hoy = fechaLocal()
    fechaRegistro.value = hoy
    fechaInicioMembresia.value = hoy
    fechaFinMembresia.value = sumarDias(hoy, 30)
    try {
      await inicializarBaseDeDatos()
      tiposMembresia.value = await listarTiposMembresia()
      precioInscripcion.value = await obtenerPrecioInscripcion()
      const mensual = tiposMembresia.value.find(p => p.nombre.trim().toLocaleLowerCase('es-MX') === 'mensual')
      if (mensual) tipoMembresiaId.value = String(mensual.id)
      else if (tiposMembresia.value.length) tipoMembresiaId.value = String(tiposMembresia.value[0].id)
    } catch (error) {
      mostrarMensaje('No se pudo inicializar la base de datos local.', 'error')
    }
  })

  onBeforeUnmount(() => { pausarLectorGlobal(false).catch(console.error) })

  return {
    tiposMembresia, tipoMembresiaId, incluirInscripcion, precioInscripcion,
    nombre, telefono, telefonoEmergencia, fechaRegistro, fechaInicioMembresia, fechaFinMembresia,
    sinHuella, template1, template2, capturandoHuella, guardando, mensaje, tipoMensaje, mostrarMensaje,
    ...camera, capturarHuellaCliente, guardar
  }
}
