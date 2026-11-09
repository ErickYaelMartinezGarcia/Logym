<script setup>
import { ref } from 'vue'
import { useVisita } from '../composables/useVisita'
import { mostrarToast } from '../services/toast'

defineProps({ bloqueada: Boolean })
const dialogo = ref(null)
const { nombre, guardando, mensaje, porConfirmar, abrir, guardar } = useVisita()
function mostrar() { abrir(); dialogo.value.showModal() }
function cerrar() { if (!guardando.value) dialogo.value?.close() }
async function confirmar() {
  const recibo = await guardar()
  if (!recibo) return
  dialogo.value?.close()
  mostrarToast({ titulo: 'Visita registrada', detalle: 'Cobro de $50.00 y una entrada registrados. No permite reingreso.', tipo: 'exito' })
}
</script>

<template>
  <button class="visita-fab" type="button" :disabled="bloqueada" aria-haspopup="dialog" aria-label="Registrar visita ocasional de 50 pesos" @click="mostrar">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M3 5h18v5a2 2 0 0 0 0 4v5H3v-5a2 2 0 0 0 0-4V5Z"/><path d="M15 5v3m0 3v2m0 3v3"/></svg>
    <span>Visita</span>
  </button>
  <dialog ref="dialogo" class="visita-modal" aria-labelledby="visita-titulo" @cancel="guardando && $event.preventDefault()">
    <form @submit.prevent="confirmar">
      <p class="visita-eyebrow">VISITA OCASIONAL</p>
      <h2 id="visita-titulo">Registrar visita</h2>
      <label for="nombre-visita">Nombre del visitante <span>(opcional)</span></label>
      <input id="nombre-visita" v-model="nombre" maxlength="120" :disabled="guardando || porConfirmar" autocomplete="off" autofocus />
      <div class="visita-resumen"><span>Visita de un día</span><strong>$50.00 <small>MXN</small></strong></div>
      <p class="visita-aviso">Una sola entrada. No permite reingreso</p>
      <p v-if="mensaje" class="visita-error" role="alert">{{ mensaje }}</p>
      <button class="visita-confirmar" :disabled="guardando">{{ guardando ? 'Registrando…' : porConfirmar ? 'Reintentar confirmación · $50' : 'Registrar visita · $50' }}</button>
      <button type="button" class="visita-cancelar" :disabled="guardando" @click="cerrar">Cancelar</button>
    </form>
  </dialog>
</template>

<style scoped>
.visita-fab { position:fixed; right:max(24px,env(safe-area-inset-right)); bottom:max(24px,env(safe-area-inset-bottom)); z-index:110; display:flex; align-items:center; gap:9px; padding:16px 23px; border:0; border-radius:17px; color:white; background:radial-gradient(circle at 0 50%,transparent 0 6px,#ee2046 6.5px); box-shadow:0 8px 24px #0006; font-weight:700; transition:transform .18s,filter .18s; }
.visita-fab svg { width:25px; height:25px; }
.visita-fab:hover:not(:disabled) { transform:translateY(-3px); filter:brightness(1.12); }
.visita-fab:disabled { opacity:.45; cursor:not-allowed; }
.visita-fab:focus-visible,.visita-modal button:focus-visible { outline:3px solid #ff8fa5; outline-offset:4px; }
.visita-modal { width:min(360px,calc(100vw - 32px)); max-height:calc(100dvh - 40px); overflow:auto; padding:25px; border:1px solid #3b3036; border-radius:20px; background:#181a1f; color:#f7f7f8; box-shadow:0 22px 80px #0008; margin:auto; }
.visita-modal::backdrop { background:#000a; }
.visita-modal form { display:grid; gap:13px; }
.visita-eyebrow { color:#ff6b85; font-size:10px; letter-spacing:.16em; font-weight:700; margin:0; }
.visita-modal h2 { font-size:23px; font-weight:800; margin:0 0 4px; }
.visita-modal label { font-size:12px; color:#d5d6dc; }
.visita-modal label span { color:#9396a1; }
.visita-modal input { width:100%; min-width:0; padding:11px; background:#101216; border:1px solid #3c4049; border-radius:9px; color:white; }
.visita-modal input:focus { outline:2px solid #ee2046; }
.visita-resumen { display:flex; align-items:center; justify-content:space-between; gap:8px; padding:12px 0; border-bottom:1px solid #34363e; font-size:13px; }
.visita-resumen strong { font-size:22px; white-space:nowrap; }
.visita-resumen small { font-size:10px; color:#aaaeb8; }
.visita-aviso { font-size:12px; color:#babdc6; }
.visita-error { font-size:12px; color:#ffb0c0; line-height:1.5; }
.visita-confirmar { padding:12px; border:0; border-radius:10px; background:#ee2046; color:white; font-weight:700; font-size:13px; }
.visita-cancelar { padding:8px; border:0; background:transparent; color:#bdc0ca; font-size:12px; }
.visita-modal button:disabled { opacity:.5; cursor:wait; }
@media(prefers-reduced-motion:reduce) { .visita-fab { transition:none; } }
@media(max-width:600px) { .visita-fab { right:16px; bottom:16px; padding:13px 18px; } }
</style>
