<script setup>
import { computed, onMounted, ref } from 'vue'
import AppShell from '../components/AppShell.vue'
import { inicializarBaseDeDatos, listarClientes, listarClientesMembresia, obtenerMetricasDashboard } from '../db'

const clientes = ref([])
const clientesMembresia = ref([])
const busqueda = ref('')
const metricas = ref([
  { titulo: 'Miembros activos', valor: '0', trend: '+8.4%', icon: '◉' },
  { titulo: 'Entradas de hoy', valor: '0', trend: 'En tiempo real', icon: '↗' },
  { titulo: 'Nuevos esta semana', valor: '0', trend: 'Últimos 7 días', icon: '+' },
  { titulo: 'Huellas activas', valor: '0', trend: 'Biometría lista', icon: '⌁' }
])

function formatearFecha(valor) {
  if (!valor) return 'Sin fecha'
  const soloFecha = String(valor).slice(0, 10)
  if (!/^\d{4}-\d{2}-\d{2}$/.test(soloFecha)) return valor
  const [anio, mes, dia] = soloFecha.split('-')
  return `${dia}/${mes}/${anio}`
}

function etiquetaEstado(estado) {
  if (estado === 'vencida') return 'Vencida'
  if (estado === 'por_vencer') return 'Por vencer'
  if (estado === 'sin_fecha') return 'Sin fecha'
  return 'Activa'
}

const clientesFiltrados = computed(() => {
  const termino = busqueda.value.trim().toLowerCase()
  if (!termino) return clientesMembresia.value
  return clientesMembresia.value.filter((cliente) => {
    const nombre = String(cliente.nombre || '').toLowerCase()
    const telefono = String(cliente.telefono || '').toLowerCase()
    const id = String(cliente.id || '')
    return nombre.includes(termino) || telefono.includes(termino) || id.includes(termino)
  })
})

const alertasMembresia = computed(() => clientesMembresia.value.filter((cliente) => cliente.estado_membresia === 'vencida' || cliente.estado_membresia === 'por_vencer'))

const membresiasVencidas = computed(() => clientesMembresia.value.filter((cliente) => cliente.estado_membresia === 'vencida').length)
const membresiasPorVencer = computed(() => clientesMembresia.value.filter((cliente) => cliente.estado_membresia === 'por_vencer').length)

onMounted(async () => {
  try {
    await inicializarBaseDeDatos()
    clientes.value = await listarClientes()
    clientesMembresia.value = await listarClientesMembresia()
    const datos = await obtenerMetricasDashboard()
    metricas.value[0].valor = String(datos.total_clientes || 0)
    metricas.value[1].valor = String(datos.asistencias_hoy || 0)
    metricas.value[2].valor = String(datos.nuevos_semana || 0)
    metricas.value[3].valor = String(datos.huella_activas || 0)
  } catch (error) {
    console.error('Error al cargar dashboard:', error)
  }
})
</script>

<template>
  <AppShell>
    <div class="dashboard-head" style="display:flex; justify-content:space-between; gap:24px; align-items:end; margin-bottom:32px">
      <div><p class="eyebrow">CENTRO DE CONTROL / 01</p><h1 class="page-title">Tu gimnasio,<br><span style="color:#ee2046">en movimiento.</span></h1><p class="muted" style="margin:15px 0 0">Una vista clara de lo que está pasando hoy en VIRTUS.</p></div>
      <div class="surface" style="padding:13px 16px; min-width:168px"><span class="status-dot" style="display:inline-block; margin-right:8px"></span><span style="font-size:12px">Operación normal</span><div class="muted" style="font-size:11px; margin-top:5px">Última sincronización: ahora</div></div>
    </div>

    <section class="metrics-grid" style="display:grid; grid-template-columns:repeat(auto-fit,minmax(190px,1fr)); gap:14px; margin-bottom:25px">
      <article v-for="item in metricas" :key="item.titulo" class="surface" style="padding:21px">
        <div style="display:flex; justify-content:space-between; color:#ee2046; font-size:20px"><span>{{ item.icon }}</span><span class="muted" style="font-size:10px; letter-spacing:.08em">{{ item.trend }}</span></div>
        <p class="muted" style="font-size:12px; margin:25px 0 7px">{{ item.titulo }}</p><strong style="font-size:31px; letter-spacing:-.04em">{{ item.valor }}</strong>
      </article>
    </section>

    <section class="surface" style="padding:24px; margin-bottom:20px">
      <div style="display:flex; justify-content:space-between; align-items:end; gap:14px; margin-bottom:16px">
        <div><p class="eyebrow">MEMBRESÍAS CRÍTICAS</p><h2 style="margin:7px 0 0; font-size:20px">Vencidas y próximas a caducar</h2></div>
        <div class="alert-counters">
          <span class="counter-pill danger">Vencidas: {{ membresiasVencidas }}</span>
          <span class="counter-pill warning">Próximas 7 días: {{ membresiasPorVencer }}</span>
        </div>
      </div>
      <div style="overflow-x:auto">
        <table style="width:100%; border-collapse:collapse; text-align:left; font-size:13px">
          <thead>
            <tr class="muted">
              <th style="padding:13px 8px">MIEMBRO</th>
              <th style="padding:13px 8px">TELÉFONO</th>
              <th style="padding:13px 8px">INICIO</th>
              <th style="padding:13px 8px">FIN</th>
              <th style="padding:13px 8px">ESTADO</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cliente in alertasMembresia" :key="`alerta-${cliente.id}`" style="border-top:1px solid #292d33">
              <td style="padding:14px 8px"><strong>{{ cliente.nombre }}</strong></td>
              <td class="muted" style="padding:14px 8px">{{ cliente.telefono || 'Sin teléfono' }}</td>
              <td class="muted" style="padding:14px 8px">{{ formatearFecha(cliente.fecha_inicio_membresia) }}</td>
              <td class="muted" style="padding:14px 8px">{{ formatearFecha(cliente.fecha_fin_membresia) }}</td>
              <td style="padding:14px 8px"><span class="status-chip" :class="`status-${cliente.estado_membresia}`">{{ etiquetaEstado(cliente.estado_membresia) }}</span></td>
            </tr>
            <tr v-if="!alertasMembresia.length"><td colspan="5" class="muted" style="padding:23px 8px; text-align:center">No hay membresías vencidas o por vencer en los próximos 7 días.</td></tr>
          </tbody>
        </table>
      </div>
    </section>

    <div class="dashboard-columns" style="display:grid; grid-template-columns:minmax(0,1.4fr) minmax(280px,.8fr); gap:18px">
      <section class="surface" style="padding:24px">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:18px"><div><p class="eyebrow">BASE DE MIEMBROS</p><h2 style="margin:7px 0 0; font-size:20px">Altas recientes</h2></div><span class="muted" style="font-size:12px">Últimos 20</span></div>
        <div class="neon-line" style="margin-bottom:4px"></div>
        <div style="overflow-x:auto"><table style="width:100%; border-collapse:collapse; text-align:left; font-size:13px"><thead><tr class="muted"><th style="padding:14px 8px">MIEMBRO</th><th style="padding:14px 8px">TELÉFONO</th><th style="padding:14px 8px">ALTA</th></tr></thead><tbody><tr v-for="cliente in clientes" :key="cliente.id" style="border-top:1px solid #292d33"><td style="padding:15px 8px"><strong>{{ cliente.nombre }}</strong><div class="muted" style="font-size:11px; margin-top:3px">ID #{{ cliente.id }}</div></td><td class="muted" style="padding:15px 8px">{{ cliente.telefono || 'Sin teléfono' }}</td><td class="muted" style="padding:15px 8px">{{ cliente.fecha_registro }}</td></tr><tr v-if="!clientes.length"><td colspan="3" class="muted" style="padding:28px 8px; text-align:center">Aún no hay miembros registrados.</td></tr></tbody></table></div>
      </section>
      <section class="surface" style="padding:24px"><p class="eyebrow">RITMO SEMANAL</p><h2 style="margin:7px 0 4px; font-size:20px">Asistencias</h2><p class="muted" style="font-size:12px; margin-top:0">Actividad estimada de los últimos 7 días</p><div style="height:190px; display:flex; align-items:end; gap:9px; margin-top:25px"><div v-for="(height,index) in [36,52,44,71,57,86,63]" :key="index" :style="{height: `${height}%`}" style="flex:1; border-radius:7px 7px 2px 2px; background:linear-gradient(#ee2046,#85142e); box-shadow:0 0 12px rgba(238,32,70,.2)"></div></div><div style="display:flex; justify-content:space-between; color:#747a84; font-size:10px; margin-top:10px"><span>LUN</span><span>MAR</span><span>MIÉ</span><span>JUE</span><span>VIE</span><span>SÁB</span><span>DOM</span></div></section>
    </div>

    <section class="surface" style="padding:24px; margin-top:18px">
      <div style="display:flex; justify-content:space-between; align-items:end; gap:14px; margin-bottom:14px">
        <div><p class="eyebrow">BUSCADOR DE CLIENTES</p><h2 style="margin:7px 0 0; font-size:20px">Registro y vencimiento de membresía</h2></div>
        <input v-model="busqueda" class="search-input" type="search" placeholder="Buscar por nombre, teléfono o ID" />
      </div>
      <div style="overflow-x:auto">
        <table style="width:100%; border-collapse:collapse; text-align:left; font-size:13px">
          <thead>
            <tr class="muted">
              <th style="padding:13px 8px">MIEMBRO</th>
              <th style="padding:13px 8px">REGISTRO</th>
              <th style="padding:13px 8px">INICIO</th>
              <th style="padding:13px 8px">FIN</th>
              <th style="padding:13px 8px">ESTADO</th>
              <th style="padding:13px 8px">DÍAS</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cliente in clientesFiltrados" :key="`busqueda-${cliente.id}`" style="border-top:1px solid #292d33">
              <td style="padding:14px 8px"><strong>{{ cliente.nombre }}</strong><div class="muted" style="font-size:11px; margin-top:3px">ID #{{ cliente.id }} · {{ cliente.telefono || 'Sin teléfono' }}</div></td>
              <td class="muted" style="padding:14px 8px">{{ formatearFecha(cliente.fecha_registro) }}</td>
              <td class="muted" style="padding:14px 8px">{{ formatearFecha(cliente.fecha_inicio_membresia) }}</td>
              <td class="muted" style="padding:14px 8px">{{ formatearFecha(cliente.fecha_fin_membresia) }}</td>
              <td style="padding:14px 8px"><span class="status-chip" :class="`status-${cliente.estado_membresia}`">{{ etiquetaEstado(cliente.estado_membresia) }}</span></td>
              <td class="muted" style="padding:14px 8px">{{ cliente.estado_membresia === 'sin_fecha' ? '-' : cliente.dias_restantes }}</td>
            </tr>
            <tr v-if="!clientesFiltrados.length"><td colspan="6" class="muted" style="padding:23px 8px; text-align:center">No se encontraron clientes con ese criterio.</td></tr>
          </tbody>
        </table>
      </div>
    </section>
  </AppShell>
</template>

<style scoped>
.alert-counters { display:flex; gap:9px; flex-wrap:wrap; }
.counter-pill { display:inline-flex; align-items:center; padding:6px 10px; border-radius:999px; font-size:11px; font-weight:700; }
.counter-pill.danger { color:#ff7a93; background:#3c1721; border:1px solid #6e1d30; }
.counter-pill.warning { color:#f6d873; background:#352b10; border:1px solid #60501b; }
.status-chip { display:inline-flex; align-items:center; padding:5px 9px; border-radius:999px; font-size:11px; font-weight:700; border:1px solid transparent; }
.status-vencida { color:#ff7a93; border-color:#6e1d30; background:#3c1721; }
.status-por_vencer { color:#f6d873; border-color:#60501b; background:#352b10; }
.status-activa { color:#6de1a9; border-color:#1b5e44; background:#103024; }
.status-sin_fecha { color:#9aa1ab; border-color:#414750; background:#22262d; }
.search-input { min-width:270px; border:1px solid #343941; border-radius:10px; padding:10px 12px; color:#fff; background:#111317; }
.search-input:focus { outline:0; border-color:#ee2046; box-shadow:0 0 0 3px rgba(238,32,70,.12); }
@media (max-width: 800px) {
  .dashboard-head { align-items: start !important; flex-direction: column; }
  .dashboard-columns { grid-template-columns: 1fr !important; }
  .search-input { width:100%; min-width:0; }
}
</style>
