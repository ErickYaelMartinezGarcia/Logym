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
  <div class="pointer-events-none fixed right-3 top-5 z-[500] flex w-[min(440px,calc(100vw-24px))] flex-col gap-5 sm:right-6 sm:top-7" aria-label="Notificaciones">
    <TransitionGroup name="toast">
      <article v-for="aviso in avisos" :key="aviso.id"
        class="toast-card pointer-events-auto relative isolate mt-3 min-h-[116px] overflow-visible rounded-[24px] border bg-gradient-to-br px-5 pb-5 pl-[105px] pt-5 text-white shadow-[0_16px_38px_rgba(0,0,0,.45)] sm:pl-[116px]"
        :class="[temas[aviso.tipo]?.fondo || temas.info.fondo, temas[aviso.tipo]?.borde || temas.info.borde]"
        role="status" aria-live="polite" @mouseenter="pausarToast(aviso.id)" @mouseleave="reanudarToast(aviso.id)"
        @focusin="pausarToast(aviso.id)" @focusout="reanudarToast(aviso.id)">
        <div class="pointer-events-none absolute inset-0 overflow-hidden rounded-[23px]" aria-hidden="true">
          <svg class="absolute bottom-[-9px] left-[-4px] h-[105px] w-[110px] opacity-35" viewBox="0 0 120 120" fill="currentColor">
            <path d="M60 112C27 94 14 68 21 30c-5 33 9 59 39 82Zm-7-16C34 91 23 78 20 63c17 2 29 13 33 33ZM37 74C20 69 12 58 11 43c16 4 25 15 26 31Zm-7-27C18 38 17 26 21 16c12 9 16 19 9 31Zm35 65c33-18 46-44 39-82 5 33-9 59-39 82Zm7-16c19-5 30-18 33-33-17 2-29 13-33 33Zm16-22c17-5 25-16 26-31-16 4-25 15-26 31Zm7-27c12-9 13-21 9-31-12 9-16 19-9 31Z"/>
            <path d="M46 83V56l14-18 15 6 8 17-9-4-7 10v17l-9 9-12-10Zm14-35-8 10 12 4 7-9-11-5ZM43 39l17-13 21 13-6 9-15-10-12 11-5-10Z"/>
          </svg>
          <svg class="absolute bottom-0 right-0 h-[65px] w-[110px] opacity-20" viewBox="0 0 110 65" fill="currentColor"><path d="M0 65C34 25 63 49 110 8V65Z"/></svg>
          <svg class="absolute bottom-0 left-[85px] h-[35px] w-[120px] opacity-15" viewBox="0 0 120 35" fill="none" stroke="currentColor"><path d="M0 35Q55-30 120 35"/></svg>
        </div>
        <div class="absolute -left-1 -top-5 z-10 grid h-[69px] w-[69px] place-items-center rounded-full border-2 border-white/30 text-[39px] font-extrabold shadow-[0_7px_17px_rgba(0,0,0,.25)]"
          :class="(temas[aviso.tipo] || temas.info).insignia" aria-hidden="true">{{ (temas[aviso.tipo] || temas.info).icono }}</div>
        <div class="relative z-10 min-w-0">
          <h3 class="pr-4 text-[21px] font-bold leading-tight tracking-tight">{{ aviso.titulo }}</h3>
          <p v-if="aviso.nombre" class="mt-1 text-sm font-semibold">{{ aviso.nombre }}</p>
          <p v-if="aviso.detalle" class="mt-1 text-[13px] leading-relaxed text-white/90">{{ aviso.detalle }}</p>
        </div>
        <button type="button" class="absolute right-3 top-3 z-20 grid h-8 w-8 place-items-center rounded-full text-2xl text-white/90 hover:bg-white/15 focus-visible:outline focus-visible:outline-2 focus-visible:outline-white"
          aria-label="Cerrar notificación" @click="cerrarToast(aviso.id)">×</button>
      </article>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-enter-active,.toast-leave-active{transition:opacity .22s,transform .22s}
.toast-enter-from,.toast-leave-to{opacity:0;transform:translateY(-14px)}
</style>
