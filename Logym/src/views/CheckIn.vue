<script setup>
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { useRouter } from 'vue-router'
import AppShell from '../components/AppShell.vue'
import { inicializarBaseDeDatos } from '../db'

const router = useRouter()
const mensaje = ref('Coloca tu huella para validar tu acceso')
const cargando = ref(false)
async function capturarHuella() {
  try { cargando.value = true; const template = await invoke('capturar_huella'); mensaje.value = `Lectura recibida: ${template.slice(0, 18)}...` } catch (error) { mensaje.value = 'No se pudo leer el dispositivo biométrico.' } finally { cargando.value = false }
}
onMounted(async () => { try { await inicializarBaseDeDatos() } catch (error) { mensaje.value = 'No se pudo abrir la base de datos local.' } })
</script>

<template>
  <AppShell>
    <div class="checkin-layout">
      <section class="checkin-copy"><p class="eyebrow">CONTROL DE ACCESO / 00</p><h1 class="page-title">Listo para<br><span style="color:#ee2046">entrenar?</span></h1><p class="muted" style="max-width:390px; margin-top:17px; line-height:1.6">Identifica tu huella y entra a VIRTUS sin filas, sin tarjetas y sin complicaciones.</p><div class="neon-line" style="max-width:230px; margin-top:30px"></div></section>
      <section class="surface checkin-card"><div class="fingerprint-orb"><div class="fingerprint-ring ring-one"></div><div class="fingerprint-ring ring-two"></div><span>⌁</span></div><p class="checkin-message">{{ mensaje }}</p><button class="primary-button" :disabled="cargando" @click="capturarHuella">{{ cargando ? 'Leyendo dispositivo...' : 'Leer mi huella' }} <span>→</span></button><button class="text-button" @click="router.push('/registro')">¿Nuevo miembro? Regístralo aquí</button></section>
    </div>
  </AppShell>
</template>

<style scoped>
.checkin-layout { display:grid; grid-template-columns:1fr minmax(320px,430px); align-items:center; gap:12%; min-height:calc(100vh - 160px); }.checkin-card { padding:42px 38px; text-align:center; }.fingerprint-orb { position:relative; display:grid; place-items:center; width:180px; height:180px; margin:0 auto 26px; border:1px solid #5a1b2a; border-radius:50%; color:#ee2046; background:radial-gradient(circle,rgba(238,32,70,.2),rgba(238,32,70,.02) 58%,transparent 60%); box-shadow:0 0 60px rgba(238,32,70,.14); }.fingerprint-orb span { font-size:78px; line-height:1; transform:rotate(90deg); text-shadow:0 0 25px #ee2046; }.fingerprint-ring { position:absolute; border:1px solid rgba(238,32,70,.27); border-radius:50%; }.ring-one { inset:14px; }.ring-two { inset:29px; }.checkin-message { color:#c8cbd0; font-size:14px; margin:0 0 22px; }.text-button { margin-top:19px; border:0; color:#8e949e; background:transparent; font-size:12px; }.text-button:hover { color:#ee2046; }@media(max-width:800px){.checkin-layout{grid-template-columns:1fr; min-height:auto; gap:35px}.checkin-copy{text-align:center}.checkin-copy .neon-line{margin-left:auto;margin-right:auto}.checkin-card{padding:30px 22px}}
</style>
