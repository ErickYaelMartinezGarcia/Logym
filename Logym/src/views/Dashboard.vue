<script setup>
import AppShell from '../components/AppShell.vue'
import { useDashboard } from '../composables/useDashboard'

const {
  clientes, busqueda, clientesFiltrados, resumenAsistencias, metricas,
  alertasMembresia, membresiasVencidas, membresiasPorVencer
} = useDashboard()

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

</script>

<template>
  <AppShell>
    <div class="dashboard-head">
      <div><p class="eyebrow">CENTRO DE CONTROL / 01</p><h1 class="page-title">Tu gimnasio,<br><span class="accent-text">en movimiento.</span></h1><p class="muted page-intro__description">Una vista clara de lo que está pasando hoy en VIRTUS.</p></div>
      <div class="surface dashboard-status"><span class="status-dot mr-2 inline-block"></span><span class="dashboard-status__label">Operación normal</span><div class="muted dashboard-status__detail">Última sincronización: ahora</div></div>
    </div>
    <section class="surface page-section mt-[18px]">
      <div class="section-heading">
        <div><p class="eyebrow">BUSCADOR DE CLIENTES</p><h2 class="section-title">Registro y vencimiento de membresía</h2></div>
        <input v-model="busqueda" class="search-input" type="search" maxlength="80" placeholder="Escribe nombre, teléfono o ID para buscar" aria-label="Buscar cliente por nombre, teléfono o ID" />
      </div>
      <div class="table-scroll">
        <table class="data-table">
          <thead>
            <tr class="muted">
              <th>MIEMBRO</th><th>REGISTRO</th><th>INICIO</th><th>FIN</th><th>ESTADO</th><th>DÍAS</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cliente in clientesFiltrados" :key="`busqueda-${cliente.id}`">
              <td><strong>{{ cliente.nombre }}</strong><div class="muted secondary">ID #{{ cliente.id }} · {{ cliente.telefono || 'Sin teléfono' }}</div></td>
              <td class="muted">{{ formatearFecha(cliente.fecha_registro) }}</td><td class="muted">{{ formatearFecha(cliente.fecha_inicio_membresia) }}</td><td class="muted">{{ formatearFecha(cliente.fecha_fin_membresia) }}</td>
              <td><span class="status-chip" :class="`status-${cliente.estado_membresia}`">{{ etiquetaEstado(cliente.estado_membresia) }}</span></td>
              <td class="muted">{{ cliente.estado_membresia === 'sin_fecha' ? '-' : cliente.dias_restantes }}</td>
            </tr>
            <tr v-if="!busqueda.trim()"><td colspan="6" class="muted empty-state">Escribe un nombre, teléfono o ID para consultar un cliente.</td></tr>
            <tr v-else-if="!clientesFiltrados.length"><td colspan="6" class="muted empty-state">No se encontraron clientes con ese criterio.</td></tr>
          </tbody>
        </table>
      </div>
    </section>

    <section class="metrics-grid">
      <article v-for="item in metricas" :key="item.titulo" class="surface metric-card">
        <div class="metric-card__top"><span>{{ item.icon }}</span><span class="muted metric-card__trend">{{ item.trend }}</span></div>
        <p class="muted metric-card__label">{{ item.titulo }}</p><strong class="metric-card__value">{{ item.valor }}</strong>
      </article>
    </section>

    <section class="surface page-section">
      <div class="section-heading">
        <div><p class="eyebrow">ASISTENCIAS</p><h2 class="section-title">Ritmo real del gimnasio</h2></div>
        <span class="muted dashboard-panel__heading-meta">Ventana móvil de 7 días</span>
      </div>
      <div class="attendance-summary">
        <article class="attendance-item">
          <span class="muted">Entradas últimos 7 días</span>
          <strong>{{ resumenAsistencias.ultimos7Dias }}</strong>
        </article>
        <article class="attendance-item">
          <span class="muted">Día más concurrido</span>
          <strong>{{ resumenAsistencias.diaMasConcurrido }}</strong>
        </article>
        <article class="attendance-item">
          <span class="muted">Hora de mayor afluencia</span>
          <strong>{{ resumenAsistencias.horaMayorAfluencia }}</strong>
        </article>
      </div>
    </section>

    <section class="surface page-section">
      <div class="section-heading">
        <div><p class="eyebrow">MEMBRESÍAS CRÍTICAS</p><h2 class="section-title">Vencidas y próximas a caducar</h2></div>
        <div class="alert-counters">
          <span class="counter-pill danger">Vencidas: {{ membresiasVencidas }}</span>
          <span class="counter-pill warning">Próximas 7 días: {{ membresiasPorVencer }}</span>
        </div>
      </div>
      <div class="table-scroll"><table class="data-table">
          <thead>
            <tr class="muted">
              <th>MIEMBRO</th><th>TELÉFONO</th><th>INICIO</th><th>FIN</th><th>ESTADO</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="cliente in alertasMembresia" :key="`alerta-${cliente.id}`">
              <td><strong>{{ cliente.nombre }}</strong></td><td class="muted">{{ cliente.telefono || 'Sin teléfono' }}</td><td class="muted">{{ formatearFecha(cliente.fecha_inicio_membresia) }}</td><td class="muted">{{ formatearFecha(cliente.fecha_fin_membresia) }}</td>
              <td><span class="status-chip" :class="`status-${cliente.estado_membresia}`">{{ etiquetaEstado(cliente.estado_membresia) }}</span></td>
            </tr>
            <tr v-if="!alertasMembresia.length"><td colspan="5" class="muted empty-state">No hay membresías vencidas o por vencer en los próximos 7 días.</td></tr>
          </tbody>
        </table>
      </div>
    </section>

    <div class="dashboard-columns">
      <section class="surface dashboard-panel">
        <div class="dashboard-panel__heading"><div><p class="eyebrow">BASE DE MIEMBROS</p><h2 class="dashboard-panel__heading-title">Altas recientes</h2></div><span class="muted dashboard-panel__heading-meta--small">Últimos 20</span></div>
        <div class="neon-line mb-1"></div>
        <div class="table-scroll"><table class="data-table"><thead><tr class="muted"><th>MIEMBRO</th><th>TELÉFONO</th><th>ALTA</th></tr></thead><tbody><tr v-for="cliente in clientes" :key="cliente.id"><td><strong>{{ cliente.nombre }}</strong><div class="muted secondary">ID #{{ cliente.id }}</div></td><td class="muted">{{ cliente.telefono || 'Sin teléfono' }}</td><td class="muted">{{ cliente.fecha_registro }}</td></tr><tr v-if="!clientes.length"><td colspan="3" class="muted empty-state">Aún no hay miembros registrados.</td></tr></tbody></table></div>
      </section>
      <section class="surface dashboard-panel"><p class="eyebrow">RITMO SEMANAL</p><h2 class="dashboard-panel__heading-title">Asistencias</h2><p class="muted dashboard-panel__heading-meta">Los datos se calculan a partir de accesos válidos, sin duplicar visitas del mismo cliente en un día.</p><div class="attendance-note"><span class="status-dot"></span><span>Una huella repetida el mismo día no genera otra entrada.</span></div></section>
    </div>
  </AppShell>
</template>

<style scoped>
.alert-counters { display:flex; gap:9px; flex-wrap:wrap; }
.counter-pill { display:inline-flex; align-items:center; padding:6px 10px; border-radius:999px; font-size:11px; font-weight:700; }
.counter-pill.danger { color:#ff7a93; background:#3c1721; border:1px solid #6e1d30; }
.counter-pill.warning { color:#f6d873; background:#352b10; border:1px solid #60501b; }
.attendance-summary { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:12px; }
.attendance-item { display:flex; flex-direction:column; gap:12px; padding:16px; border:1px solid #292d33; border-radius:10px; background:#111317; }
.attendance-item span { font-size:12px; }
.attendance-item strong { font-size:24px; }
.attendance-note { display:flex; align-items:center; gap:9px; margin-top:28px; padding:13px; border:1px solid #292d33; border-radius:9px; color:#c8cbd0; font-size:12px; }
.status-chip { display:inline-flex; align-items:center; padding:5px 9px; border-radius:999px; font-size:11px; font-weight:700; border:1px solid transparent; }
.status-vencida { color:#ff7a93; border-color:#6e1d30; background:#3c1721; }
.status-por_vencer { color:#f6d873; border-color:#60501b; background:#352b10; }
.status-activa { color:#6de1a9; border-color:#1b5e44; background:#103024; }
.status-sin_fecha { color:#9aa1ab; border-color:#414750; background:#22262d; }
.search-input { width:min(390px,100%); min-width:320px; border:1px solid #343941; border-radius:10px; padding:10px 12px; color:#fff; background:#111317; }
.search-input:focus { outline:0; border-color:#ee2046; box-shadow:0 0 0 3px rgba(238,32,70,.12); }
@media (max-width: 800px) {
  .dashboard-head { align-items: start !important; flex-direction: column; }
  .dashboard-columns { grid-template-columns: 1fr !important; }
  .attendance-summary { grid-template-columns: 1fr; }
  .search-input { width:100%; min-width:0; }
}
</style>
