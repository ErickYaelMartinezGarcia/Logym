import { ref } from 'vue'
import { registrarVisita } from '../services/tauriApi'

const CLAVE_PENDIENTE = 'logym.visita.pendiente'
// Solo se conserva el ID, nunca el nombre en almacenamiento del navegador.
function recuperarPendiente() {
  try {
    const id = typeof localStorage === 'undefined' ? null : localStorage.getItem(CLAVE_PENDIENTE)
    return /^[a-f0-9]{32}$/.test(id || '') ? { id, nombre: '' } : null
  } catch (error) { console.error('No se pudo consultar la confirmación de visita pendiente:', error); return null }
}
let pendiente = recuperarPendiente()

export function useVisita() {
  const nombre = ref(pendiente?.nombre || '')
  const guardando = ref(false)
  const mensaje = ref('')
  const porConfirmar = ref(Boolean(pendiente))

  function abrir() {
    nombre.value = pendiente?.nombre || ''
    porConfirmar.value = Boolean(pendiente)
    mensaje.value = pendiente ? 'Hay una confirmación pendiente. Reintenta para comprobarla sin duplicar el cobro.' : ''
  }

  async function guardar() {
    if (guardando.value) return null
    const limpio = nombre.value.trim()
    if (limpio.length > 120 || /[\u0000-\u001f\u007f]/.test(limpio)) {
      mensaje.value = 'El nombre debe tener máximo 120 caracteres y no contener caracteres de control.'
      return null
    }
    guardando.value = true
    mensaje.value = ''
    try {
      if (!pendiente) {
        const bytes = crypto.getRandomValues(new Uint8Array(16))
        pendiente = { id: Array.from(bytes, b => b.toString(16).padStart(2, '0')).join(''), nombre: limpio }
      }
      if (typeof localStorage !== 'undefined') localStorage.setItem(CLAVE_PENDIENTE, pendiente.id)
      porConfirmar.value = true
      const recibo = await registrarVisita(pendiente.id, pendiente.nombre)
      if (typeof localStorage !== 'undefined') localStorage.removeItem(CLAVE_PENDIENTE)
      pendiente = null
      porConfirmar.value = false
      nombre.value = ''
      return recibo
    } catch (error) {
      console.error('Error al registrar visita ocasional:', error)
      mensaje.value = 'No se pudo confirmar la visita. Reintenta esta misma operación para evitar un cobro duplicado.'
      return null
    } finally {
      guardando.value = false
    }
  }
  return { nombre, guardando, mensaje, porConfirmar, abrir, guardar }
}
