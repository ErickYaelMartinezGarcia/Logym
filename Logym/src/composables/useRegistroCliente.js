import { onMounted, ref } from 'vue'
import { capturarHuella, compararHuella, guardarCliente, inicializarBaseDeDatos } from '../services/tauriApi'
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

  function sumarDias(fechaISO, dias) {
    const fecha = new Date(`${fechaISO}T00:00:00`)
    fecha.setDate(fecha.getDate() + dias)
    return fecha.toISOString().slice(0, 10)
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
      fechaRegistro.value = new Date().toISOString().slice(0, 10)
      fechaInicioMembresia.value = fechaRegistro.value
      fechaFinMembresia.value = sumarDias(fechaInicioMembresia.value, 30)
      mostrarMensaje('Miembro registrado. Ya puede utilizar el check-in.', 'success')
    } catch (error) {
      console.error('Error al guardar miembro:', error)
      mostrarMensaje('No se pudo guardar el miembro. Revisa la base de datos.', 'error')
    }
  }

  onMounted(async () => {
    const hoy = new Date().toISOString().slice(0, 10)
    fechaRegistro.value = hoy
    fechaInicioMembresia.value = hoy
    fechaFinMembresia.value = sumarDias(hoy, 30)
    try {
      await inicializarBaseDeDatos()
    } catch (error) {
      mostrarMensaje('No se pudo inicializar la base de datos local.', 'error')
    }
  })

  return {
    nombre, telefono, telefonoEmergencia, fechaRegistro, fechaInicioMembresia, fechaFinMembresia,
    template1, template2, mensaje, tipoMensaje, mostrarMensaje,
    ...camera, capturarHuellaCliente, guardar
  }
}
