<script setup>
import { useToasts } from '../services/toast'
const { avisos, cerrarToast, pausarToast, reanudarToast } = useToasts()
// Detecta el recurso del proyecto sin inventar su extensión.
import espartano from '../assets/espartano_laurel.svg'
const temas = {
  exito: { fondo: 'from-[#0bbf68] via-[#049451] to-[#006b3c]', icono: '✓', insignia: 'bg-[#006d40]' },
  aviso: { fondo: 'from-[#ff9e19] via-[#ef7909] to-[#c65b05]', icono: '!', insignia: 'bg-[#b94c07]' },
  error: { fondo: 'from-[#f43b65] via-[#d91e4c] to-[#a60f38]', icono: 'x', insignia: 'bg-[#8c1640]' },
  info: { fondo: 'from-[#258dff] via-[#0969d9] to-[#064caa]', icono: 'i', insignia: 'bg-[#0756b8]' }
}
</script>

<template>
  <div class="pointer-events-none fixed right-3 top-5 z-[500] flex w-[min(340px,calc(100vw-24px))] flex-col gap-3 sm:right-5 sm:top-5" aria-label="Notificaciones">
    <TransitionGroup name="toast">
      <article v-for="aviso in avisos" :key="aviso.id"
        class="toast-card pointer-events-auto relative isolate mt-2 min-h-[88px] overflow-visible rounded-[17px] bg-gradient-to-br px-3 pb-3 pl-[78px] pt-3 text-white shadow-[0_16px_38px_rgba(0,0,0,.45)] sm:pl-[82px]"
        :class="temas[aviso.tipo]?.fondo || temas.info.fondo"
        role="status" aria-live="polite" @mouseenter="pausarToast(aviso.id)" @mouseleave="reanudarToast(aviso.id)"
        @focusin="pausarToast(aviso.id)" @focusout="reanudarToast(aviso.id)">
        <div class="pointer-events-none absolute inset-0 overflow-hidden rounded-[16px]" aria-hidden="true">
          <img v-if="espartano" :src="espartano" alt="" class="absolute bottom-1 left-2 h-[68px] w-[68px] object-contain opacity-30" />
          <svg class="absolute bottom-0 right-0 h-[65px] w-[110px] opacity-20" viewBox="0 0 110 65" fill="currentColor"><path d="M0 65C34 25 63 49 110 8V65Z"/></svg>
          <svg class="absolute bottom-0 left-[85px] h-[35px] w-[120px] opacity-15" viewBox="0 0 120 35" fill="none" stroke="currentColor"><path d="M0 35Q55-30 120 35"/></svg>
        </div>
        <div class="absolute -left-1 -top-3 z-10 grid h-[35px] w-[35px] place-items-center rounded-full text-[28px] font-extrabold shadow-[0_7px_17px_rgba(0,0,0,.25)]"
          :class="(temas[aviso.tipo] || temas.info).insignia" aria-hidden="true">{{ (temas[aviso.tipo] || temas.info).icono }}</div>
        <div class="relative z-10 min-w-0">
          <h3 class="pr-5 text-[15px] font-bold leading-tight tracking-tight">{{ aviso.titulo }}</h3>
          <p v-if="aviso.nombre" class="mt-1 text-xs font-semibold">{{ aviso.nombre }}</p>
          <p v-if="aviso.detalle" class="mt-1 text-[11px] leading-relaxed text-white/90">{{ aviso.detalle }}</p>
        </div>
        <button type="button" class="absolute right-2 top-2 z-20 grid h-7 w-7 place-items-center rounded-full text-2xl text-white/90 hover:bg-white/15 focus-visible:outline focus-visible:outline-2 focus-visible:outline-white"
          aria-label="Cerrar notificación" @click="cerrarToast(aviso.id)">×</button>
      </article>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-enter-active,.toast-leave-active{transition:opacity .22s,transform .22s}
.toast-enter-from,.toast-leave-to{opacity:0;transform:translateY(-14px)}
</style>
