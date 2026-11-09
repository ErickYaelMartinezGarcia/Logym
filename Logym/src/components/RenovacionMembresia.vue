<script setup>
import { ref, watch, nextTick } from 'vue'
import { useRenovacion } from '../composables/useRenovacion'
import { duracionPlan } from '../services/duracionPlanes'
const { abierta, cliente, planes, planId, plan, cargando, guardando, error, pendiente, abrir, cerrar, confirmar } = useRenovacion()
const dialogo = ref(null)
const moneda = n => ((n || 0) / 100).toLocaleString('es-MX', { style: 'currency', currency: 'MXN' })
watch(abierta, async valor => { await nextTick(); if (valor && !dialogo.value.open) dialogo.value.showModal(); else if (!valor) dialogo.value.close() })
</script>
<template>
  <button v-if="pendiente && !abierta" class="renewal-pending" @click="abrir(null)">Revisar renovación pendiente</button>
  <dialog ref="dialogo" class="renewal-dialog" aria-labelledby="renovacion-titulo" @cancel.prevent="cerrar" @close="cerrar">
    <form @submit.prevent="confirmar">
      <h2 id="renovacion-titulo">Renovar membresía</h2>
      <p><strong>{{ cliente?.nombre }}</strong> · ID #{{ cliente?.id }}</p>
      <p class="muted">Vigencia actual: {{ cliente?.fecha_inicio_membresia || '—' }} → {{ cliente?.fecha_fin_membresia || '—' }}</p>
      <p v-if="cargando" role="status">Consultando planes y vigencia…</p>
      <label for="plan-renovacion">Plan para renovar</label>
      <select id="plan-renovacion" v-model="planId" :disabled="cargando || guardando || !!pendiente">
        <option v-for="p in planes" :key="p.id" :value="String(p.id)">{{ p.nombre }} · {{ duracionPlan(p) }} — {{ moneda(p.precio_centavos) }}</option>
      </select>
      <p v-if="!pendiente">Total a cobrar: <strong>{{ moneda(plan?.precio_centavos) }}</strong>.</p>
      <p class="muted">Se conservan los días pagados. Si ya venció, la nueva vigencia comienza hoy.</p>
      <p v-if="pendiente" role="status">Confirmación pendiente: el reintento recupera la misma renovación y su cobro original.</p>
      <p v-if="error" role="alert" class="renewal-error">{{ error }}</p>
      <div class="renewal-buttons"><button type="button" :disabled="guardando || cargando" @click="cerrar">Cerrar</button><button :disabled="guardando || cargando || (!plan && !pendiente)">{{ guardando ? 'Confirmando…' : pendiente ? 'Reintentar confirmación' : 'Confirmar renovación y cobro' }}</button></div>
    </form>
  </dialog>
</template>
<style scoped>
.renewal-dialog{width:min(520px,calc(100% - 32px));max-height:90vh;overflow:auto;border:1px solid #414853;border-radius:16px;padding:24px;background:#171a20;color:#fff}.renewal-dialog::backdrop{background:#000b}.renewal-dialog form{display:grid;gap:14px}.renewal-dialog h2{font-size:23px}.renewal-dialog select,.renewal-buttons button{padding:12px;border:1px solid #454b54;border-radius:9px;background:#232830;color:#fff;min-width:0}.renewal-buttons{display:flex;gap:10px;justify-content:flex-end;flex-wrap:wrap}.renewal-buttons button:last-child{background:#275b97}.renewal-buttons button:disabled{opacity:.5}.renewal-error{color:#ffb1bc}.renewal-pending{position:fixed;bottom:16px;left:16px;z-index:130;padding:12px;border-radius:9px;background:#75551d;color:#fff}
</style>
