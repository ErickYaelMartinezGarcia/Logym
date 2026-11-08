import { ref } from 'vue'

// Canal global reutilizable. El temporizador se pausa mientras el usuario
// interactúa con el toast y continúa al retirar el cursor.
const avisos = ref([])
let siguienteId = 0
const temporizadores = new Map()
function programar(id, ms) {
  if (ms <= 0) return
  const entrada = { restante: ms, inicio: Date.now(), timer: setTimeout(() => cerrarToast(id), ms) }
  temporizadores.set(id, entrada)
}
export function cerrarToast(id) {
  avisos.value = avisos.value.filter(a => a.id !== id)
  clearTimeout(temporizadores.get(id)?.timer)
  temporizadores.delete(id)
}
export function pausarToast(id) {
  const entrada = temporizadores.get(id)
  if (!entrada || entrada.pausado) return
  clearTimeout(entrada.timer)
  entrada.restante = Math.max(0, entrada.restante - (Date.now() - entrada.inicio))
  entrada.pausado = true
}
export function reanudarToast(id) {
  const entrada = temporizadores.get(id)
  if (!entrada || !entrada.pausado) return
  entrada.pausado = false
  entrada.inicio = Date.now()
  entrada.timer = setTimeout(() => cerrarToast(id), entrada.restante)
}
export function mostrarToast({ titulo, detalle = '', tipo = 'info', nombre = '', duracion = 6000 }) {
  const id = ++siguienteId
  if (avisos.value.length >= 4) cerrarToast(avisos.value[0].id)
  avisos.value = [...avisos.value, { id, titulo, detalle, tipo, nombre, duracion }]
  programar(id, duracion)
  return id
}
export function useToasts() { return { avisos, mostrarToast, cerrarToast, pausarToast, reanudarToast } }
