<script setup>
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { useRouter } from 'vue-router'
import AppShell from '../components/AppShell.vue'
import { buscarClientePorHuella, inicializarBaseDeDatos, renovarMembresiaCliente } from '../db'

const router = useRouter()
const mensaje = ref('Coloca tu huella para validar tu acceso')
const cargando = ref(false)
const procesandoRenovacion = ref(false)
const clienteDetectado = ref(null)
const diasPlan = ref(30)
const montoRenovacion = ref('')

function formatearFecha(valor) {
  if (!valor) return 'Sin fecha'
  const fecha = String(valor).slice(0, 10)
  if (!/^\d{4}-\d{2}-\d{2}$/.test(fecha)) return valor
  const [anio, mes, dia] = fecha.split('-')
  return `${dia}/${mes}/${anio}`
}

function etiquetaEstado(estado) {
  if (estado === 'vencida') return 'Membresía vencida'
  if (estado === 'por_vencer') return 'Membresía por vencer'
  if (estado === 'activa') return 'Membresía activa'
  return 'Sin fecha de membresía'
}

function resumenDias(cliente) {
  if (!cliente) return ''
  if (cliente.estado_membresia === 'vencida') return `Venció hace ${Math.abs(cliente.dias_restantes)} día(s)`
  if (cliente.estado_membresia === 'por_vencer') return `Vence en ${cliente.dias_restantes} día(s)`
  if (cliente.estado_membresia === 'activa') return `${cliente.dias_restantes} día(s) restantes`
  return 'Configura fechas de membresía'
}

async function capturarHuella() {
  try {
    cargando.value = true
    const template = await invoke('capturar_huella')
    const cliente = await buscarClientePorHuella(template)
    if (!cliente) {
      clienteDetectado.value = null
      mensaje.value = 'Huella leída, pero no coincide con un miembro registrado.'
      return
    }

    clienteDetectado.value = cliente
    if (cliente.estado_membresia === 'vencida') {
      mensaje.value = 'Membresía vencida detectada. Puedes renovarla desde este panel.'
    } else if (cliente.estado_membresia === 'por_vencer') {
      mensaje.value = 'Membresía próxima a vencer. Se recomienda renovar hoy.'
    } else {
      mensaje.value = `Acceso válido: ${cliente.nombre}`
    }
  } catch (error) {
    console.error('Error en check-in:', error)
    mensaje.value = 'No se pudo leer el dispositivo biométrico.'
  } finally {
    cargando.value = false
  }
}

async function renovarMembresia() {
  if (!clienteDetectado.value) return
  const dias = Number(diasPlan.value)
  const monto = Number(montoRenovacion.value || 0)
  if (!Number.isInteger(dias) || dias <= 0) {
    mensaje.value = 'Ingresa una duración válida en días para renovar.'
    return
  }

  try {
    procesandoRenovacion.value = true
    const actualizado = await renovarMembresiaCliente(clienteDetectado.value.id, dias, monto)
    clienteDetectado.value = actualizado
    mensaje.value = `Membresía renovada para ${actualizado.nombre}. Nuevo inicio: ${formatearFecha(actualizado.fecha_inicio_membresia)}.`
  } catch (error) {
    console.error('Error al renovar:', error)
    mensaje.value = 'No se pudo renovar la membresía. Intenta de nuevo.'
  } finally {
    procesandoRenovacion.value = false
  }
}

onMounted(async () => {
  try {
    await inicializarBaseDeDatos()
  } catch (error) {
    mensaje.value = 'No se pudo abrir la base de datos local.'
  }
})
</script>

<template>
  <AppShell>
    <div class="checkin-layout">
      <section class="checkin-copy"><p class="eyebrow">CONTROL DE ACCESO / 00</p><h1 class="page-title">Listo para<br><span style="color:#ee2046">entrenar?</span></h1><p class="muted" style="max-width:390px; margin-top:17px; line-height:1.6">Identifica tu huella y entra a VIRTUS sin filas, sin tarjetas y sin complicaciones.</p><div class="neon-line" style="max-width:230px; margin-top:30px"></div></section>
      <section class="surface checkin-card"><div class="fingerprint-orb"><div class="fingerprint-ring ring-one"></div><div class="fingerprint-ring ring-two"></div><span>⌁</span></div><p class="checkin-message">{{ mensaje }}</p><button class="primary-button" :disabled="cargando" @click="capturarHuella">{{ cargando ? 'Leyendo dispositivo...' : 'Leer mi huella' }} <span>→</span></button><button class="text-button" @click="router.push('/registro')">¿Nuevo miembro? Regístralo aquí</button>
        <div v-if="clienteDetectado" class="cliente-panel">
          <div class="panel-head">
            <img v-if="clienteDetectado.foto_base64" :src="clienteDetectado.foto_base64" alt="Foto del cliente" class="cliente-foto" />
            <div class="cliente-info">
              <strong>{{ clienteDetectado.nombre }}</strong>
              <small>ID #{{ clienteDetectado.id }} · {{ clienteDetectado.telefono || 'Sin teléfono' }}</small>
              <span class="estado-chip" :class="`estado-${clienteDetectado.estado_membresia}`">{{ etiquetaEstado(clienteDetectado.estado_membresia) }}</span>
            </div>
          </div>

          <div class="fechas-grid">
            <div><small>Registro</small><p>{{ formatearFecha(clienteDetectado.fecha_registro) }}</p></div>
            <div><small>Inicio</small><p>{{ formatearFecha(clienteDetectado.fecha_inicio_membresia) }}</p></div>
            <div><small>Fin</small><p>{{ formatearFecha(clienteDetectado.fecha_fin_membresia) }}</p></div>
          </div>
          <p class="resumen-dias">{{ resumenDias(clienteDetectado) }}</p>

          <div class="renovacion-box">
            <p class="muted" style="margin:0 0 8px">Cobro y renovación inmediata</p>
            <div class="renovacion-grid">
              <label>Días<input v-model.number="diasPlan" type="number" min="1" /></label>
              <label>Monto<input v-model="montoRenovacion" type="number" min="0" step="0.01" placeholder="0.00" /></label>
            </div>
            <button class="secondary-button" :disabled="procesandoRenovacion" @click="renovarMembresia">{{ procesandoRenovacion ? 'Procesando renovación...' : 'Renovar membresía hoy' }}</button>
          </div>
        </div>
      </section>
    </div>
  </AppShell>
</template>

<style scoped>
.checkin-layout { display:grid; grid-template-columns:1fr minmax(320px,430px); align-items:center; gap:12%; min-height:calc(100vh - 160px); }
.checkin-card { padding:42px 38px; text-align:center; }
.fingerprint-orb { position:relative; display:grid; place-items:center; width:180px; height:180px; margin:0 auto 26px; border:1px solid #5a1b2a; border-radius:50%; color:#ee2046; background:radial-gradient(circle,rgba(238,32,70,.2),rgba(238,32,70,.02) 58%,transparent 60%); box-shadow:0 0 60px rgba(238,32,70,.14); }
.fingerprint-orb span { font-size:78px; line-height:1; transform:rotate(90deg); text-shadow:0 0 25px #ee2046; }
.fingerprint-ring { position:absolute; border:1px solid rgba(238,32,70,.27); border-radius:50%; }
.ring-one { inset:14px; }
.ring-two { inset:29px; }
.checkin-message { color:#c8cbd0; font-size:14px; margin:0 0 22px; }
.text-button { margin-top:19px; border:0; color:#8e949e; background:transparent; font-size:12px; }
.text-button:hover { color:#ee2046; }
.cliente-panel { margin-top:20px; padding:14px; border:1px solid #343941; border-radius:12px; text-align:left; background:#121519; }
.panel-head { display:flex; gap:12px; align-items:center; }
.cliente-foto { width:60px; height:60px; object-fit:cover; border-radius:10px; border:1px solid #4c5260; background:#0e1013; }
.cliente-info strong { display:block; font-size:14px; }
.cliente-info small { display:block; margin-top:4px; color:#8f96a1; font-size:11px; }
.estado-chip { display:inline-flex; margin-top:8px; padding:4px 9px; border-radius:999px; border:1px solid transparent; font-size:10px; font-weight:700; }
.estado-vencida { color:#ff7a93; border-color:#6e1d30; background:#3c1721; }
.estado-por_vencer { color:#f6d873; border-color:#60501b; background:#352b10; }
.estado-activa { color:#6de1a9; border-color:#1b5e44; background:#103024; }
.estado-sin_fecha { color:#9aa1ab; border-color:#414750; background:#22262d; }
.fechas-grid { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:8px; margin-top:14px; }
.fechas-grid small { color:#8f96a1; font-size:10px; }
.fechas-grid p { margin:5px 0 0; font-size:12px; }
.resumen-dias { margin:12px 0 0; font-size:12px; color:#d3d7de; }
.renovacion-box { margin-top:12px; padding-top:12px; border-top:1px solid #2e333b; }
.renovacion-grid { display:grid; grid-template-columns:1fr 1fr; gap:8px; margin-bottom:10px; }
.renovacion-grid label { color:#9da3ad; font-size:11px; }
.renovacion-grid input { display:block; width:100%; margin-top:5px; padding:9px 10px; border:1px solid #343941; border-radius:8px; outline:0; color:#fff; background:#111317; }
.secondary-button { width:100%; padding:11px 12px; border:1px solid #353a42; border-radius:10px; color:#fff; background:#1a1d22; font-size:12px; font-weight:700; }
.secondary-button:hover { border-color:#ee2046; }
.secondary-button:disabled { opacity:.55; cursor:not-allowed; }
@media(max-width:800px){
  .checkin-layout{grid-template-columns:1fr; min-height:auto; gap:35px}
  .checkin-copy{text-align:center}
  .checkin-copy .neon-line{margin-left:auto;margin-right:auto}
  .checkin-card{padding:30px 22px}
  .fechas-grid{grid-template-columns:1fr}
  .renovacion-grid{grid-template-columns:1fr}
}
</style>
