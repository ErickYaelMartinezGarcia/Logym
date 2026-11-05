<script setup>
import { useToasts } from '../services/toast'
const { avisos, cerrarToast } = useToasts()
const estilos = {
  exito: 'border-emerald-500 bg-[#142b22] text-emerald-300',
  aviso: 'border-amber-400 bg-[#302719] text-amber-200',
  error: 'border-rose-500 bg-[#301a22] text-rose-200',
  info: 'border-sky-400 bg-[#172534] text-sky-200'
}
</script>

<template>
  <div class="pointer-events-none fixed right-3 top-3 z-[500] flex w-[min(420px,calc(100vw-24px))] flex-col gap-3 sm:right-6 sm:top-6" aria-label="Notificaciones">
    <TransitionGroup name="toast">
      <div v-for="aviso in avisos" :key="aviso.id" class="pointer-events-auto relative overflow-hidden rounded-xl border border-l-4 p-4 pr-11 shadow-2xl" :class="estilos[aviso.tipo] || estilos.info" role="status" aria-live="polite">
        <strong class="block text-sm font-bold">{{ aviso.titulo }}</strong>
        <span v-if="aviso.nombre" class="mt-1 block text-sm text-white">{{ aviso.nombre }}</span>
        <p v-if="aviso.detalle" class="mt-1 text-xs leading-relaxed text-slate-300">{{ aviso.detalle }}</p>
        <button type="button" class="absolute right-3 top-2 text-xl text-slate-300 hover:text-white" aria-label="Cerrar notificación" @click="cerrarToast(aviso.id)">×</button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-enter-active,.toast-leave-active{transition:opacity .2s,transform .2s}
.toast-enter-from,.toast-leave-to{opacity:0;transform:translateY(-12px)}
</style>
