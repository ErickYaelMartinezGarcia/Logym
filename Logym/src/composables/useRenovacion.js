import { computed, ref } from 'vue'
import { listarClientesMembresia, listarTiposMembresia, renovarMembresia } from '../services/tauriApi'
import { mensajeErrorRenovacion } from '../services/mensajesLector'
import { mostrarToast } from '../services/toast'
import { ordenarPlanes } from '../services/duracionPlanes'

const clave = 'logym.renovacion.pendiente'
function recuperar() {
  try {
    const p = JSON.parse(localStorage.getItem(clave) || 'null')
    return p && /^[a-f0-9]{32}$/.test(p.id) && Number.isSafeInteger(p.clienteId) && p.clienteId > 0 && Number.isSafeInteger(p.planId) && p.planId > 0 ? p : null
  } catch (e) { console.error('Recuperación de confirmación de renovación:', e); return null }
}
const pendiente = ref(recuperar())
const abierta = ref(false), cliente = ref(null), planes = ref([]), planId = ref('')
const cargando = ref(false), guardando = ref(false), error = ref(''), ultimoResultado = ref(null)
const plan = computed(() => planes.value.find(p => String(p.id) === planId.value))

export function useRenovacion() {
  async function abrir(persona) {
    if (guardando.value || cargando.value || abierta.value) return
    abierta.value = true; error.value = ''; cargando.value = true
    cliente.value = persona; planes.value = []; planId.value = ''
    try {
      const [catalogo, clientes] = await Promise.all([listarTiposMembresia(), listarClientesMembresia()])
      const id = pendiente.value?.clienteId ?? persona?.id
      cliente.value = clientes.find(c => c.id === id) || { id, nombre: `Cliente #${id}` }
      planes.value = ordenarPlanes(catalogo.filter(p => !p.solo_alta && p.activo !== false))
      planId.value = String(pendiente.value?.planId ?? planes.value[0]?.id ?? '')
      if (!pendiente.value && !cliente.value.activo) { error.value = 'El cliente no está disponible para renovar.'; planId.value = '' }
    } catch (e) { console.error('Preparación de renovación:', e); error.value = 'No se pudieron cargar los datos. Cierra e intenta nuevamente.' }
    finally { cargando.value = false }
  }
  function cerrar() { if (!guardando.value && !cargando.value) abierta.value = false }
  async function confirmar() {
    if (!abierta.value || guardando.value || cargando.value || !cliente.value || (!pendiente.value && !plan.value)) return null
    guardando.value = true; error.value = ''
    try {
      if (!pendiente.value) {
        const p = { id: crypto.randomUUID().replaceAll('-', ''), clienteId: cliente.value.id, planId: Number(planId.value) }
        // Persistir antes de enviar: después de reiniciar solo se reenvía la misma operación.
        localStorage.setItem(clave, JSON.stringify(p))
        pendiente.value = p
      }
      const p = pendiente.value
      const resultado = await renovarMembresia(p.clienteId, p.planId, p.id)
      localStorage.removeItem(clave)
      pendiente.value = null; ultimoResultado.value = resultado; abierta.value = false
      mostrarToast({ titulo: 'Membresía renovada', detalle: 'El cobro y la vigencia se actualizaron correctamente.', tipo: 'exito' })
      return resultado
    } catch (e) {
      console.error('Confirmación de renovación:', e)
      // Estos dos rechazos ocurren antes de cualquier escritura en Rust. Un
      // error técnico o una respuesta perdida nunca autorizan otro ID.
      if (['Selecciona una membresía vigente', 'Cliente no encontrado o desactivado'].includes(e instanceof Error ? e.message : e)) {
        try { localStorage.removeItem(clave); pendiente.value = null }
        catch (storageError) { console.error('Conservación de confirmación pendiente:', storageError) }
      }
      error.value = pendiente.value ? 'No se pudo confirmar el resultado. Reintenta esta misma operación para evitar otro cobro.' : mensajeErrorRenovacion(e)
      return null
    } finally { guardando.value = false }
  }
  return { abierta, cliente, planes, planId, plan, cargando, guardando, error, pendiente, ultimoResultado, abrir, cerrar, confirmar }
}
