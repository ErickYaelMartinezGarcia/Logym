import { onMounted, onBeforeUnmount, ref } from 'vue'
import { capturarHuella, compararHuella, guardarCliente, inicializarBaseDeDatos, pausarLectorGlobal } from '../services/tauriApi'
import { useCamera } from './useCamera'

export function useRegistroCliente() {
  const nombre = ref('')
  const telefono = ref('')
  const telefonoEmergencia = ref('')
  const fechaRegistro = ref('')
  const fechaInicioMembresia = ref('')
  const fechaFinMembresia = ref('')
  const template1 = ref('')
  const template2 = ref('')
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

  const camera = useCamera(mostrarMensaje)

  async function capturarHuellaCliente(numero) {
    try {
      const template = await capturarHuella()
      if (numero === 1) template1.value = template
      else template2.value = template
      mostrarMensaje(`Huella ${numero} capturada. Repite para confirmar la identidad.`, 'success')
    } catch (error) {
      console.error(`Error al capturar huella ${numero}:`, error)
      mostrarMensaje('No se pudo capturar la huella. Verifica que el lector y su controlador estén conectados.', 'error')
    }
  }

  async function guardar() {
    if (!nombre.value.trim() || !template1.value || !template2.value) {
      mostrarMensaje('Completa el nombre y las dos capturas de huella.', 'error')
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
    const telefonoValido = (valor) => valor.trim().length <= 25 && /^[0-9+() -]*$/.test(valor.trim())
    if (nombre.value.trim().length > 120 || [...nombre.value].some(ch => ch.charCodeAt(0) < 32 || ch.charCodeAt(0) === 127)) {
      mostrarMensaje('El nombre debe tener máximo 120 caracteres y no contener caracteres de control.', 'error')
      return
    }
    if (!telefonoValido(telefono.value) || !telefonoValido(telefonoEmergencia.value)) {
      mostrarMensaje('Introduce teléfonos válidos de máximo 25 caracteres.', 'error')
      return
    }
    try {
      const huellasCoinciden = await compararHuella(template1.value, template2.value)
      if (!huellasCoinciden) {
        mostrarMensaje('Las capturas no coinciden. Repite el registro biométrico.', 'error')
        return
      }
      await guardarCliente({
        nombre: nombre.value.trim(),
        telefono: telefono.value.trim(),
        telefonoEmergencia: telefonoEmergencia.value.trim(),
        fechaRegistro: fechaRegistro.value,
        fechaInicioMembresia: fechaInicioMembresia.value,
        fechaFinMembresia: fechaFinMembresia.value,
        fotoBase64: camera.fotoBase64.value,
        templateHuella: template1.value
      })
      nombre.value = ''
      telefono.value = ''
      telefonoEmergencia.value = ''
      template1.value = ''
      template2.value = ''
      camera.fotoBase64.value = ''
      fechaRegistro.value = fechaLocal()
      fechaInicioMembresia.value = fechaRegistro.value
      fechaFinMembresia.value = sumarDias(fechaInicioMembresia.value, 30)
      mostrarMensaje('Miembro registrado. Ya puede utilizar el check-in.', 'success')
    } catch (error) {
      console.error('Error al guardar miembro:', error)
      mostrarMensaje('No se pudo guardar el miembro. Revisa la base de datos.', 'error')
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
    } catch (error) {
      mostrarMensaje('No se pudo inicializar la base de datos local.', 'error')
    }
  })

  onBeforeUnmount(() => { pausarLectorGlobal(false).catch(console.error) })

  return {
    nombre, telefono, telefonoEmergencia, fechaRegistro, fechaInicioMembresia, fechaFinMembresia,
    template1, template2, mensaje, tipoMensaje, mostrarMensaje,
    ...camera, capturarHuellaCliente, guardar
  }
}
