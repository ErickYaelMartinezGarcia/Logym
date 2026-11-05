<script setup>
import AppShell from '../components/AppShell.vue'
import espartano from '../assets/espartano_laurel.svg'
import { ref, computed, onMounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { obtenerAsistenciasDiarias } from '../services/tauriApi'
import { onBeforeUnmount } from 'vue'
onBeforeUnmount(() => unsubscribeIngresos?.())
import { useDashboard } from '../composables/useDashboard'

const {
  clientes, busqueda, clientesFiltrados, resumenAsistencias, metricas,
  alertasMembresia, membresiasVencidas, membresiasPorVencer
} = useDashboard()

const asistenciasHoy = computed(() => metricas.value.find(item => item.titulo === 'Entradas de hoy')?.valor ?? '0')
const miembrosActivos = computed(() => metricas.value.find(item => item.titulo === 'Miembros activos')?.valor ?? '0')
const periodoGrafica = ref('semana')
const asistenciasDiarias = ref([])
const errorGrafica = ref('')
const diasGrafica = computed(() => {
  const hoy = new Date()
  const conteos = new Map(asistenciasDiarias.value.map(d => [d.fecha, Number(d.total)]))
  if (periodoGrafica.value === 'mes') {
    return Array.from({ length: 12 }, (_, mes) => {
      const fecha = [hoy.getFullYear(), String(mes + 1).padStart(2, '0')].join('-')
      return { fecha, etiqueta: new Date(hoy.getFullYear(), mes, 1).toLocaleDateString('es-MX', { month: 'short' }), total: conteos.get(fecha) ?? 0 }
    })
  }
  const inicio = new Date(hoy.getFullYear(), hoy.getMonth(), hoy.getDate())
  if (periodoGrafica.value === 'semana') inicio.setDate(inicio.getDate() - 6)
  const resultado = []
  for (let fecha = new Date(inicio); fecha <= hoy; fecha.setDate(fecha.getDate() + 1)) {
    const clave = [fecha.getFullYear(), String(fecha.getMonth() + 1).padStart(2, '0'), String(fecha.getDate()).padStart(2, '0')].join('-')
    resultado.push({ fecha: clave, etiqueta: fecha.toLocaleDateString('es-MX', { weekday: 'short' }), total: conteos.get(clave) ?? 0 })
  }
  return resultado
})
const maxAsistencias = computed(() => Math.max(1, ...diasGrafica.value.map(d => d.total)))
const puntosGrafica = computed(() => diasGrafica.value.map((d, i, arr) => ({
  ...d,
  x: arr.length === 1 ? 50 : 6 + i * 88 / (arr.length - 1),
  y: 88 - d.total / maxAsistencias.value * 74
})))
const lineaGrafica = computed(() => puntosGrafica.value.map(p => p.x + ',' + p.y).join(' '))
const areaGrafica = computed(() => puntosGrafica.value.length ? '6,92 ' + lineaGrafica.value + ' 94,92' : '')
const topDias = computed(() => [...diasGrafica.value].sort((a, b) => b.total - a.total).slice(0, 5))
async function cargarGrafica() {
  try { asistenciasDiarias.value = await obtenerAsistenciasDiarias(periodoGrafica.value); errorGrafica.value = '' }
  catch (e) { errorGrafica.value = String(e) }
}
let desuscribirGrafica
onMounted(async () => {
  await cargarGrafica()
  try { desuscribirGrafica = await listen('logym://datos-actualizados', cargarGrafica) }
  catch (e) { console.error('No se pudo escuchar actualización de asistencias', e) }
})
onBeforeUnmount(() => desuscribirGrafica?.())


function formatearFecha(valor) {
  if (!valor) return 'Sin fecha'
  const soloFecha = String(valor).slice(0, 10)
  if (!/^\d{4}-\d{2}-\d{2}$/.test(soloFecha)) return valor
  const [anio, mes, dia] = soloFecha.split('-')
  return `${dia}/${mes}/${anio}`
}

function etiquetaEstado(estado) {
  if (estado === 'pendiente') return 'Pendiente'
  if (estado === 'vencida') return 'Vencida'
  if (estado === 'por_vencer') return 'Por vencer'
  if (estado === 'sin_fecha') return 'Sin fecha'
  return 'Activa'
}

</script>

<template>
  <AppShell>
    <div class="dashboard-head relative isolate overflow-hidden rounded-2xl py-5">
      <img :src="espartano" alt="" aria-hidden="true" class="pointer-events-none absolute left-1/2 top-1/2 -z-10 h-48 w-48 -translate-x-1/2 -translate-y-1/2 object-contain opacity-[0.09] brightness-0 invert sm:h-64 sm:w-64" />
      <div>
        <p class="eyebrow">CENTRO DE CONTROL / 01</p>
        <h1 class="page-title">Tu gimnasio,<br><span class="accent-text">en movimiento.</span></h1>
        <p class="muted page-intro__description">Una vista clara de lo que está pasando hoy en VIRTUS.</p>
      </div>
      <div class="surface dashboard-status"><span class="status-dot mr-2 inline-block"></span><span
          class="dashboard-status__label">Operación normal</span>
        <div class="muted dashboard-status__detail">Última sincronización: ahora</div>
      </div>
    </div>
    <section aria-label="Resumen operativo" class="mt-6 mb-5 grid grid-cols-1 gap-5 border-y border-white/10 py-6 sm:grid-cols-3 sm:divide-x sm:divide-white/10">
      <div class="flex items-start gap-4 px-2 sm:pr-5">
        <svg class="mt-1 h-9 w-9 shrink-0 text-[#ee2046]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="9" cy="7" r="3"/><path d="M3 20v-2a6 6 0 0 1 12 0v2M17 11a3 3 0 0 0 0-6M17 15a5 5 0 0 1 4 5"/></svg>
        <div>
        <p class="text-xs font-semibold uppercase tracking-widest text-zinc-400">Asistencias hoy</p>
        <strong class="mt-2 block text-4xl font-black tracking-tight text-white">{{ asistenciasHoy }}</strong>
        <span class="mt-1 block text-xs text-zinc-500">Entradas registradas hoy</span></div>
      </div>
      <div class="flex items-start gap-4 px-2 sm:px-6">
        <svg class="mt-1 h-9 w-9 shrink-0 text-[#ee2046]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="5" width="18" height="16" rx="2"/><path d="M7 3v4M17 3v4M3 10h18M8 14h3M14 14h3"/></svg>
        <div>
        <p class="text-xs font-semibold uppercase tracking-widest text-zinc-400">Asistencias últimos 7 días</p>
        <strong class="mt-2 block text-4xl font-black tracking-tight text-white">{{ resumenAsistencias.ultimos7Dias }}</strong>
        <span class="mt-1 block text-xs text-zinc-500">Periodo móvil de siete días</span></div>
      </div>
      <div class="flex items-start gap-4 px-2 sm:pl-6">
        <svg class="mt-1 h-9 w-9 shrink-0 text-[#ee2046]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="9" cy="8" r="3"/><path d="M3 20a6 6 0 0 1 12 0M16 5a3 3 0 0 1 0 6M17 14a5 5 0 0 1 4 5"/></svg>
        <div>
        <p class="text-xs font-semibold uppercase tracking-widest text-zinc-400">Miembros activos</p>
        <strong class="mt-2 block text-4xl font-black tracking-tight text-[#ee2046]">{{ miembrosActivos }}</strong>
        <span class="mt-1 block text-xs text-zinc-500">Clientes habilitados</span></div>
      </div>
    </section>
    <section class="surface page-section mt-[18px]">
      <div class="section-heading">
        <div>
          <p class="eyebrow">BUSCADOR DE CLIENTES</p>
          <h2 class="section-title">Registro y vencimiento de membresía</h2>
        </div>
        <input v-model="busqueda" class="search-input" type="search" maxlength="80"
          placeholder="Escribe nombre, teléfono o ID para buscar"
          aria-label="Buscar cliente por nombre, teléfono o ID" />
      </div>
      <div class="table-scroll">
        <table class="data-table">
          <thead>
            <tr class="muted">
              <th>MIEMBRO</th>
              <th>PLAN</th>
              <th>REGISTRO</th>
              <th>INICIO</th>
              <th>FIN</th>
              <th>ESTADO</th>
              <th>DÍAS</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cliente in clientesFiltrados" :key="`busqueda-${cliente.id}`">
              <td><strong>{{ cliente.nombre }}</strong>
                <div class="muted secondary">ID #{{ cliente.id }} · {{ cliente.telefono || 'Sin teléfono' }}</div>
              </td>
              <td><span class="inline-flex rounded-md bg-[#ee2046]/10 px-2.5 py-1 text-xs font-semibold text-[#ff6b85]">{{ cliente.tipo_membresia }}</span></td>
              <td class="muted">{{ formatearFecha(cliente.fecha_registro) }}</td>
              <td class="muted">{{ formatearFecha(cliente.fecha_inicio_membresia) }}</td>
              <td class="muted">{{ formatearFecha(cliente.fecha_fin_membresia) }}</td>
              <td><span class="status-chip" :class="`status-${cliente.estado_membresia}`">{{
                etiquetaEstado(cliente.estado_membresia) }}</span></td>
              <td class="muted">{{ cliente.estado_membresia === 'sin_fecha' ? '-' : cliente.dias_restantes }}</td>
            </tr>
            <tr v-if="!busqueda.trim()">
              <td colspan="6" class="muted empty-state">Escribe un nombre, teléfono o ID para consultar un cliente.</td>
            </tr>
            <tr v-else-if="!clientesFiltrados.length">
              <td colspan="6" class="muted empty-state">No se encontraron clientes con ese criterio.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <section class="surface page-section">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <p class="eyebrow">ASISTENCIAS</p>
          <h2 class="section-title">Actividad del gimnasio</h2>
        </div>
        <div class="flex gap-1 rounded-lg bg-black/30 p-1" role="group" aria-label="Periodo de asistencias">
          <button v-for="opcion in [{ id: 'dia', nombre: 'Hoy' }, { id: 'semana', nombre: 'Semana' }, { id: 'mes', nombre: 'Meses' }]"
            :key="opcion.id" type="button" @click="periodoGrafica = opcion.id; cargarGrafica()"
            :aria-pressed="periodoGrafica === opcion.id"
            class="rounded-md px-3 py-1.5 text-xs font-semibold"
            :class="periodoGrafica === opcion.id ? 'bg-[#ee2046] text-white' : 'text-zinc-400 hover:bg-white/10 hover:text-white'">{{ opcion.nombre }}</button>
        </div>
      </div>
      <p v-if="errorGrafica" role="alert" class="mt-3 text-xs text-red-300">{{ errorGrafica }}</p>
      <div class="mt-4 grid grid-cols-1 gap-5 lg:grid-cols-4">
        <div class="min-w-0 lg:col-span-3">
          <svg class="h-64 w-full sm:h-72" viewBox="0 0 100 100" preserveAspectRatio="none" role="img"
            :aria-label="'Tendencia de asistencias: ' + diasGrafica.map(d => d.etiqueta + ' ' + d.total).join(', ')">
            <defs><linearGradient id="virtus-asistencias" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="#ee2046" stop-opacity=".35"/><stop offset="100%" stop-color="#ee2046" stop-opacity="0"/></linearGradient></defs>
            <path v-for="y in [14, 40, 66, 92]" :key="y" :d="'M 0 ' + y + ' H 100'" stroke="#ffffff" stroke-opacity=".09" stroke-width=".3" stroke-dasharray="1 1"/>
            <polygon v-if="puntosGrafica.length > 1" :points="areaGrafica" fill="url(#virtus-asistencias)"/>
            <polyline :points="lineaGrafica" fill="none" stroke="#ee2046" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round" vector-effect="non-scaling-stroke"/>
            <circle v-for="p in puntosGrafica" :key="p.fecha" :cx="p.x" :cy="p.y" r="1.4" fill="#ee2046" stroke="#fff" stroke-width=".45"><title>{{ p.etiqueta }}: {{ p.total }}</title></circle>
          </svg>
          <div class="mt-1 flex justify-between gap-1 text-[10px] text-zinc-400">
            <span v-for="dia in diasGrafica" :key="dia.fecha" class="min-w-0 flex-1 truncate text-center" :title="dia.fecha">{{ dia.etiqueta }}</span>
          </div>
        </div>
        <aside class="border-t border-white/10 pt-3 lg:border-l lg:border-t-0 lg:pl-5 lg:pt-0">
          <h3 class="mb-3 text-sm font-semibold text-white">Días destacados</h3>
          <div v-for="dia in topDias" :key="dia.fecha" class="mb-3 flex items-center gap-2 text-xs">
            <span class="w-10 shrink-0 capitalize text-zinc-400">{{ dia.etiqueta }}</span>
            <div class="h-2 min-w-0 flex-1 overflow-hidden rounded-full bg-white/10">
              <div class="h-full rounded-full bg-[#ee2046]" :style="{ width: (dia.total / maxAsistencias * 100) + '%' }"></div>
            </div>
            <strong class="w-6 text-right text-white">{{ dia.total }}</strong>
          </div>
          <p class="mt-3 text-xs text-zinc-500">Total: {{ diasGrafica.reduce((n, d) => n + d.total, 0) }} asistencias</p>
        </aside>
      </div>
    </section>

    <section class="surface page-section">
      <div class="section-heading">
        <div>
          <p class="eyebrow">MEMBRESÍAS CRÍTICAS</p>
          <h2 class="section-title">Vencidas y próximas a caducar</h2>
        </div>
        <div class="alert-counters">
          <span class="counter-pill danger">Vencidas: {{ membresiasVencidas }}</span>
          <span class="counter-pill warning">Próximas 7 días: {{ membresiasPorVencer }}</span>
        </div>
      </div>
      <div class="table-scroll">
        <table class="data-table">
          <thead>
            <tr class="muted">
              <th>MIEMBRO</th>
              <th>TELÉFONO</th>
              <th>INICIO</th>
              <th>FIN</th>
              <th>ESTADO</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cliente in alertasMembresia" :key="`alerta-${cliente.id}`">
              <td><strong>{{ cliente.nombre }}</strong></td>
              <td class="muted">{{ cliente.telefono || 'Sin teléfono' }}</td>
              <td class="muted">{{ formatearFecha(cliente.fecha_inicio_membresia) }}</td>
              <td class="muted">{{ formatearFecha(cliente.fecha_fin_membresia) }}</td>
              <td><span class="status-chip" :class="`status-${cliente.estado_membresia}`">{{
                etiquetaEstado(cliente.estado_membresia) }}</span></td>
            </tr>
            <tr v-if="!alertasMembresia.length">
              <td colspan="5" class="muted empty-state">No hay membresías vencidas o por vencer en los próximos 7 días.
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <div class="mt-6">
      <section class="surface dashboard-panel">
        <div class="dashboard-panel__heading">
          <div>
            <p class="eyebrow">BASE DE MIEMBROS</p>
            <h2 class="dashboard-panel__heading-title">Altas recientes</h2>
          </div><span class="muted dashboard-panel__heading-meta--small">Últimos 20</span>
        </div>
        <div class="neon-line mb-1"></div>
        <div class="table-scroll">
          <table class="data-table">
            <thead>
              <tr class="muted">
                <th>MIEMBRO</th>
                <th>TELÉFONO</th>
                <th>ALTA</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="cliente in clientes" :key="cliente.id">
                <td><strong>{{ cliente.nombre }}</strong>
                  <div class="muted secondary">ID #{{ cliente.id }}</div>
                </td>
                <td class="muted">{{ cliente.telefono || 'Sin teléfono' }}</td>
                <td class="muted">{{ cliente.fecha_registro }}</td>
              </tr>
              <tr v-if="!clientes.length">
                <td colspan="3" class="muted empty-state">Aún no hay miembros registrados.</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

    </div>
  </AppShell>
</template>

<style scoped>
.alert-counters {
  display: flex;
  gap: 9px;
  flex-wrap: wrap;
}

.counter-pill {
  display: inline-flex;
  align-items: center;
  padding: 6px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
}

.counter-pill.danger {
  color: #ff7a93;
  background: #3c1721;
  border: 1px solid #6e1d30;
}

.counter-pill.warning {
  color: #f6d873;
  background: #352b10;
  border: 1px solid #60501b;
}

.attendance-summary {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.attendance-item {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  border: 1px solid #292d33;
  border-radius: 10px;
  background: #111317;
}

.attendance-item span {
  font-size: 12px;
}

.attendance-item strong {
  font-size: 24px;
}

.attendance-note {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-top: 28px;
  padding: 13px;
  border: 1px solid #292d33;
  border-radius: 9px;
  color: #c8cbd0;
  font-size: 12px;
}

.status-chip {
  display: inline-flex;
  align-items: center;
  padding: 5px 9px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  border: 1px solid transparent;
}

.status-vencida {
  color: #ff7a93;
  border-color: #6e1d30;
  background: #3c1721;
}

.status-por_vencer {
  color: #f6d873;
  border-color: #60501b;
  background: #352b10;
}

.status-activa {
  color: #6de1a9;
  border-color: #1b5e44;
  background: #103024;
}

.status-sin_fecha {
  color: #9aa1ab;
  border-color: #414750;
  background: #22262d;
}

.search-input {
  width: min(390px, 100%);
  min-width: 320px;
  border: 1px solid #343941;
  border-radius: 10px;
  padding: 10px 12px;
  color: #fff;
  background: #111317;
}

.search-input:focus {
  outline: 0;
  border-color: #ee2046;
  box-shadow: 0 0 0 3px rgba(238, 32, 70, .12);
}

@media (max-width: 800px) {
  .dashboard-head {
    align-items: start !important;
    flex-direction: column;
  }

  .dashboard-columns {
    grid-template-columns: 1fr !important;
  }

  .attendance-summary {
    grid-template-columns: 1fr;
  }

  .search-input {
    width: 100%;
    min-width: 0;
  }
}
</style>
