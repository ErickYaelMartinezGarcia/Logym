<script setup>
import { onMounted, ref } from 'vue'
import AppShell from '../components/AppShell.vue'
import { inicializarBaseDeDatos, listarClientes, obtenerMetricasDashboard } from '../db'

const clientes = ref([])
const metricas = ref([
  { titulo: 'Miembros activos', valor: '0', trend: '+8.4%', icon: '◉' },
  { titulo: 'Entradas de hoy', valor: '0', trend: 'En tiempo real', icon: '↗' },
  { titulo: 'Nuevos esta semana', valor: '0', trend: 'Últimos 7 días', icon: '+' },
  { titulo: 'Huellas activas', valor: '0', trend: 'Biometría lista', icon: '⌁' }
])

onMounted(async () => {
  try {
    await inicializarBaseDeDatos()
    clientes.value = await listarClientes()
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

    <div class="dashboard-columns" style="display:grid; grid-template-columns:minmax(0,1.4fr) minmax(280px,.8fr); gap:18px">
      <section class="surface" style="padding:24px">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:18px"><div><p class="eyebrow">BASE DE MIEMBROS</p><h2 style="margin:7px 0 0; font-size:20px">Altas recientes</h2></div><span class="muted" style="font-size:12px">Últimos 20</span></div>
        <div class="neon-line" style="margin-bottom:4px"></div>
        <div style="overflow-x:auto"><table style="width:100%; border-collapse:collapse; text-align:left; font-size:13px"><thead><tr class="muted"><th style="padding:14px 8px">MIEMBRO</th><th style="padding:14px 8px">TELÉFONO</th><th style="padding:14px 8px">ALTA</th></tr></thead><tbody><tr v-for="cliente in clientes" :key="cliente.id" style="border-top:1px solid #292d33"><td style="padding:15px 8px"><strong>{{ cliente.nombre }}</strong><div class="muted" style="font-size:11px; margin-top:3px">ID #{{ cliente.id }}</div></td><td class="muted" style="padding:15px 8px">{{ cliente.telefono || 'Sin teléfono' }}</td><td class="muted" style="padding:15px 8px">{{ cliente.fecha_registro }}</td></tr><tr v-if="!clientes.length"><td colspan="3" class="muted" style="padding:28px 8px; text-align:center">Aún no hay miembros registrados.</td></tr></tbody></table></div>
      </section>
      <section class="surface" style="padding:24px"><p class="eyebrow">RITMO SEMANAL</p><h2 style="margin:7px 0 4px; font-size:20px">Asistencias</h2><p class="muted" style="font-size:12px; margin-top:0">Actividad estimada de los últimos 7 días</p><div style="height:190px; display:flex; align-items:end; gap:9px; margin-top:25px"><div v-for="(height,index) in [36,52,44,71,57,86,63]" :key="index" :style="{height: `${height}%`}" style="flex:1; border-radius:7px 7px 2px 2px; background:linear-gradient(#ee2046,#85142e); box-shadow:0 0 12px rgba(238,32,70,.2)"></div></div><div style="display:flex; justify-content:space-between; color:#747a84; font-size:10px; margin-top:10px"><span>LUN</span><span>MAR</span><span>MIÉ</span><span>JUE</span><span>VIE</span><span>SÁB</span><span>DOM</span></div></section>
    </div>
  </AppShell>
</template>

<style scoped>
@media (max-width: 800px) { .dashboard-head { align-items: start !important; flex-direction: column; } .dashboard-columns { grid-template-columns: 1fr !important; } }
</style>
