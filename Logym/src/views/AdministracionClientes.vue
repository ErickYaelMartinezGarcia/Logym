<script setup>
/**
 * Administración de miembros.
 * La primera confirmación se realiza desde la tabla; la segunda exige escribir ELIMINAR.
 * Ninguna operación destructiva se ejecuta antes de confirmar ambas etapas.
 */
import { computed, onMounted, ref } from 'vue'
import AppShell from '../components/AppShell.vue'
import { listarClientesMembresia, actualizarCliente, establecerClienteActivo, eliminarClienteDefinitivamente } from '../services/tauriApi'

const clientes = ref([])
const busqueda = ref('')
const cargando = ref(false)
const mensaje = ref('')
const error = ref('')
const editando = ref(null)
const eliminando = ref(null)
const textoConfirmacion = ref('')
const procesando = ref(false)
const clientesFiltrados = computed(() => clientes.value.filter(c =>
  [c.nombre, c.telefono, c.id].some(v => String(v ?? '').toLowerCase().includes(busqueda.value.trim().toLowerCase()))
))
async function cargar() {
  cargando.value = true
  error.value = ''
  try { clientes.value = await listarClientesMembresia() }
  catch (e) { error.value = String(e) }
  finally { cargando.value = false }
}
function editar(cliente) {
  editando.value = { ...cliente }
  mensaje.value = ''
}
async function guardar() {
  if (!editando.value || procesando.value) return
  procesando.value = true
  error.value = ''
  try {
    const c = editando.value
    await actualizarCliente(c.id, {
      nombre: c.nombre,
      telefono: c.telefono || '',
      telefonoEmergencia: c.telefono_emergencia || '',
      fechaInicioMembresia: c.fecha_inicio_membresia || '',
      fechaFinMembresia: c.fecha_fin_membresia || ''
    })
    editando.value = null
    mensaje.value = 'Datos actualizados.'
    await cargar()
  } catch (e) { error.value = String(e) }
  finally { procesando.value = false }
}
async function cambiarEstado(c) {
  if (procesando.value) return
  procesando.value = true
  error.value = ''
  try {
    await establecerClienteActivo(c.id, !c.activo)
    mensaje.value = c.activo ? 'Cliente desactivado; su huella ya no permite el acceso.' : 'Cliente reactivado.'
    await cargar()
  } catch (e) { error.value = String(e) }
  finally { procesando.value = false }
}
function solicitarEliminacion(c) {
  eliminando.value = c
  textoConfirmacion.value = ''
  error.value = ''
}
async function confirmarEliminacion() {
  if (!eliminando.value || textoConfirmacion.value !== 'ELIMINAR' || procesando.value) return
  procesando.value = true
  error.value = ''
  try {
    await eliminarClienteDefinitivamente(eliminando.value.id, textoConfirmacion.value)
    eliminando.value = null
    textoConfirmacion.value = ''
    mensaje.value = 'Cliente eliminado definitivamente de la base de datos.'
    await cargar()
  } catch (e) { error.value = String(e) }
  finally { procesando.value = false }
}
onMounted(cargar)
</script>

<template>
  <AppShell>
    <section class="clientes-admin">
      <p class="eyebrow">ADMINISTRACIÓN / 03</p>
      <h1 class="page-title">Administrar <span class="accent-text">clientes.</span></h1>
      <p class="muted">Edita datos, suspende accesos o elimina definitivamente un expediente biométrico.</p>
      <div class="toolbar">
        <input v-model="busqueda" class="control" type="search" placeholder="Buscar por nombre, teléfono o ID" aria-label="Buscar clientes" />
        <button class="action" :disabled="cargando" @click="cargar">Actualizar</button>
      </div>
      <p v-if="mensaje" class="feedback success" role="status">{{ mensaje }}</p>
      <p v-if="error" class="feedback failure" role="alert">{{ error }}</p>
      <p v-if="cargando" class="muted">Cargando clientes...</p>
      <div class="table-scroll surface">
        <table class="data-table">
          <thead><tr><th>Cliente</th><th>Teléfono</th><th>Membresía</th><th>Acceso</th><th>Acciones</th></tr></thead>
          <tbody>
            <tr v-for="c in clientesFiltrados" :key="c.id">
              <td><strong>{{ c.nombre }}</strong><div class="muted">ID #{{ c.id }}</div></td>
              <td>{{ c.telefono || '—' }}</td>
              <td>{{ c.fecha_fin_membresia || 'Sin fecha' }}</td>
              <td>{{ c.activo ? 'Activo' : 'Inactivo' }}</td>
              <td><div class="actions">
                <button class="action" @click="editar(c)">Editar</button>
                <button class="action" :disabled="procesando" @click="cambiarEstado(c)">{{ c.activo ? 'Desactivar' : 'Reactivar' }}</button>
                <button class="action danger" @click="solicitarEliminacion(c)">Eliminar</button>
              </div></td>
            </tr>
            <tr v-if="!clientesFiltrados.length && !cargando"><td colspan="5" class="muted">No se encontraron clientes.</td></tr>
          </tbody>
        </table>
      </div>
      <div v-if="editando" class="overlay" @click.self="editando = null">
        <form class="dialog surface" @submit.prevent="guardar">
          <h2>Editar cliente #{{ editando.id }}</h2>
          <label>Nombre<input v-model="editando.nombre" class="control" required /></label>
          <label>Teléfono<input v-model="editando.telefono" class="control" /></label>
          <label>Teléfono de emergencia<input v-model="editando.telefono_emergencia" class="control" /></label>
          <label>Inicio de membresía<input v-model="editando.fecha_inicio_membresia" class="control" type="date" required /></label>
          <label>Fin de membresía<input v-model="editando.fecha_fin_membresia" class="control" type="date" required /></label>
          <div class="actions"><button type="button" class="action" @click="editando = null">Cancelar</button><button class="action primary" :disabled="procesando">Guardar cambios</button></div>
        </form>
      </div>
      <div v-if="eliminando" class="overlay" @click.self="eliminando = null">
        <section class="dialog surface" role="alertdialog" aria-modal="true" aria-label="Confirmar eliminación definitiva">
          <h2>Eliminar definitivamente</h2>
          <p>Primera confirmación: eliminarás a <strong>{{ eliminando.nombre }}</strong> (ID #{{ eliminando.id }}), su fotografía, su huella y sus registros asociados. Esta acción no se puede deshacer.</p>
          <label>Segunda confirmación: escribe <strong>ELIMINAR</strong>
            <input v-model="textoConfirmacion" class="control" autocomplete="off" placeholder="ELIMINAR" />
          </label>
          <div class="actions"><button class="action" @click="eliminando = null">Cancelar</button><button class="action danger" :disabled="textoConfirmacion !== 'ELIMINAR' || procesando" @click="confirmarEliminacion">Eliminar definitivamente</button></div>
        </section>
      </div>
    </section>
  </AppShell>
</template>

<style scoped>
.clientes-admin{display:grid;gap:16px;padding-bottom:28px}.toolbar,.actions{display:flex;gap:10px;flex-wrap:wrap;align-items:center}.toolbar{justify-content:space-between}.control{display:block;width:100%;min-width:0;padding:11px 13px;border:1px solid #454b54;border-radius:9px;background:#15181d;color:#fff}.toolbar .control{max-width:390px}.action{border:1px solid #454b54;border-radius:8px;background:#242930;color:#fff;padding:9px 12px;cursor:pointer}.action:disabled{opacity:.45;cursor:not-allowed}.primary{background:#266345}.danger{border-color:#a52e4b;color:#ff9eb3}.table-scroll{overflow-x:auto;padding:12px}.data-table{width:100%;border-collapse:collapse}.data-table td,.data-table th{padding:14px;text-align:left;border-bottom:1px solid #333943}.feedback{padding:12px;border-radius:8px}.success{color:#8ce8b8}.failure{color:#ff9eb3}.overlay{position:fixed;inset:0;z-index:100;background:#000b;display:grid;place-items:center;padding:20px}.dialog{width:min(520px,100%);max-height:90vh;overflow:auto;display:grid;gap:15px;padding:28px;border-radius:18px}.dialog label{display:grid;gap:7px}.dialog .actions{justify-content:flex-end}@media(max-width:700px){.toolbar .control{max-width:none}.dialog{padding:20px}}
</style>