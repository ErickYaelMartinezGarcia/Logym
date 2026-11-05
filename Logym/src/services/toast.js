import { ref } from 'vue'

// Un único canal de notificaciones para todos los módulos.
const avisos = ref([])
let siguienteId = 0
const temporizadores = new Map()
export function cerrarToast(id) {
  avisos.value = avisos.value.filter(a => a.id !== id)
  clearTimeout(temporizadores.get(id))
  temporizadores.delete(id)
}
export function mostrarToast({ titulo, detalle = '', tipo = 'info', nombre = '', duracion = 6000 }) {
  const id = ++siguienteId
  avisos.value = [...avisos.value.slice(-3), { id, titulo, detalle, tipo, nombre, duracion }]
  if (duracion > 0) temporizadores.set(id, setTimeout(() => cerrarToast(id), duracion))
  return id
}
export function useToasts() { return { avisos, mostrarToast, cerrarToast } }
