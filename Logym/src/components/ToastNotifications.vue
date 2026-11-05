<script setup>
import { useToasts } from '../services/toast'
const { avisos, cerrarToast, pausarToast, reanudarToast } = useToasts()
const temas = {
  exito: { fondo: 'from-[#0bbf68] via-[#049451] to-[#006b3c]', borde: 'border-emerald-400', icono: '✓', insignia: 'bg-[#006d40]' },
  aviso: { fondo: 'from-[#ff9e19] via-[#ef7909] to-[#c65b05]', borde: 'border-orange-300', icono: '!', insignia: 'bg-[#b94c07]' },
  error: { fondo: 'from-[#f43b65] via-[#d91e4c] to-[#a60f38]', borde: 'border-rose-400', icono: '×', insignia: 'bg-[#8c1640]' },
  info: { fondo: 'from-[#258dff] via-[#0969d9] to-[#064caa]', borde: 'border-blue-300', icono: 'i', insignia: 'bg-[#0756b8]' }
}
</script>

<template>
  <div class="pointer-events-none fixed right-3 top-5 z-[500] flex w-[min(340px,calc(100vw-24px))] flex-col gap-3 sm:right-5 sm:top-5" aria-label="Notificaciones">
    <TransitionGroup name="toast">
      <article v-for="aviso in avisos" :key="aviso.id"
        class="toast-card pointer-events-auto relative isolate mt-2 min-h-[88px] overflow-visible rounded-[17px] border bg-gradient-to-br px-3 pb-3 pl-[78px] pt-3 text-white shadow-[0_16px_38px_rgba(0,0,0,.45)] sm:pl-[82px]"
        :class="[temas[aviso.tipo]?.fondo || temas.info.fondo, temas[aviso.tipo]?.borde || temas.info.borde]"
        role="status" aria-live="polite" @mouseenter="pausarToast(aviso.id)" @mouseleave="reanudarToast(aviso.id)"
        @focusin="pausarToast(aviso.id)" @focusout="reanudarToast(aviso.id)">
        <div class="pointer-events-none absolute inset-0 overflow-hidden rounded-[16px]" aria-hidden="true">
          <!-- Silueta espartana y laureles dibujados con trazos separados. -->
          <svg class="absolute bottom-[1px] left-[5px] h-[73px] w-[75px] opacity-40" viewBox="0 0 100 100" fill="none" aria-hidden="true">
            <g stroke="currentColor" stroke-width="2.8" stroke-linecap="round">
              <path d="M48 91C19 77 13 52 24 25M52 91C81 77 87 52 76 25"/>
            </g>
            <g fill="currentColor">
              <path d="M30 73C17 74 10 65 8 56c13 1 20 7 22 17ZM23 55C11 52 8 42 10 33c11 5 15 12 13 22ZM23 38C16 28 19 18 25 12c6 10 5 18-2 26ZM37 84C25 89 16 84 12 78c11-4 19-2 25 6Z"/>
              <path d="M70 73c13 1 20-8 22-17-13 1-20 7-22 17Zm7-18c12-3 15-13 13-22-11 5-15 12-13 22Zm0-17c7-10 4-20-2-26-6 10-5 18 2 26ZM63 84c12 5 21 0 25-6-11-4-19-2-25 6Z"/>
              <path d="M50 21c-17 0-27 11-27 26v4h10l5 13 8 7v15h8V71l8-7 5-13h10v-4C77 32 67 21 50 21Zm-18 25c2-11 9-17 18-17s16 6 18 17H32Zm5 5h26l-6 9-7 5-7-5-6-9Z"/>
              <path d="M31 27c3-13 13-21 19-22 6 1 16 9 19 22l-19-9-19 9Z"/>
            </g>
          </svg>
          <svg class="absolute bottom-0 right-0 h-[65px] w-[110px] opacity-20" viewBox="0 0 110 65" fill="currentColor"><path d="M0 65C34 25 63 49 110 8V65Z"/></svg>
          <svg class="absolute bottom-0 left-[85px] h-[35px] w-[120px] opacity-15" viewBox="0 0 120 35" fill="none" stroke="currentColor"><path d="M0 35Q55-30 120 35"/></svg>
        </div>
        <div class="absolute -left-1 -top-3 z-10 grid h-[51px] w-[51px] place-items-center rounded-full border-2 border-white/30 text-[28px] font-extrabold shadow-[0_7px_17px_rgba(0,0,0,.25)]"
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
