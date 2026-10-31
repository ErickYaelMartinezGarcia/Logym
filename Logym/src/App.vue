<script setup>
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'

const aviso = ref(null)
let desuscribir = null
let ocultar = null

function cerrarAviso() {
  aviso.value = null
  if (ocultar) clearTimeout(ocultar)
}

// La respuesta de Rust conserva la causa exacta del rechazo; no ocultar errores.
function presentarResultado(resultado) {
  if (resultado === 'registrada') return { titulo: 'Acceso autorizado', detalle: 'Asistencia registrada correctamente.', tipo: 'exito', simbolo: '✓' }
  if (resultado === 'duplicada') return { titulo: 'Asistencia ya registrada', detalle: 'Este cliente ya registró su entrada hoy. No se duplicó la asistencia.', tipo: 'aviso', simbolo: '!' }
  if (resultado === 'membresia_pendiente') return { titulo: 'Membresía pendiente', detalle: 'La fecha de inicio aún no llega. Consulta la fecha de inicio del cliente.', tipo: 'aviso', simbolo: '!' }
  if (resultado === 'membresia_vencida') return { titulo: 'Membresía vencida', detalle: 'Renueva la membresía para permitir el acceso.', tipo: 'error', simbolo: '×' }
  const detalle = String(resultado || 'Error desconocido').replace(/^error:\s*/i, '').trim()
  if (detalle.includes('membresía no vigente')) return { titulo: 'Membresía no vigente', detalle: 'La fecha de inicio todavía no llega o la membresía ya terminó.', tipo: 'error', simbolo: '×' }
  if (detalle.includes('desactivado')) return { titulo: 'Cliente desactivado', detalle: 'Reactiva el cliente desde Administración para permitir su acceso.', tipo: 'error', simbolo: '×' }
  return { titulo: 'No se registró la asistencia', detalle, tipo: 'error', simbolo: '×' }
}

onMounted(async () => {
  desuscribir = await listen('logym://checkin', ({ payload }) => {
    const { cliente, resultado } = payload || {}
    if (!cliente) return
    aviso.value = { nombre: cliente.nombre, ...presentarResultado(resultado) }
    if (ocultar) clearTimeout(ocultar)
    ocultar = setTimeout(() => { aviso.value = null }, 7500)
  })
})
onBeforeUnmount(() => {
  desuscribir?.()
  if (ocultar) clearTimeout(ocultar)
})
</script>

<template>
  <div class="min-h-screen bg-[#090a0c] text-white">
    <router-view />
    <Transition name="aviso-deslizar">
      <div v-if="aviso" class="aviso-global" :class="`aviso-global--${aviso.tipo}`" role="status" aria-live="polite">
        <span class="aviso-global__icon" aria-hidden="true">{{ aviso.simbolo }}</span>
        <div class="aviso-global__contenido">
          <strong>{{ aviso.titulo }}</strong>
          <span class="aviso-global__nombre">{{ aviso.nombre }}</span>
          <p>{{ aviso.detalle }}</p>
        </div>
        <button class="aviso-global__cerrar" aria-label="Cerrar notificación" @click="cerrarAviso">×</button>
        <span class="aviso-global__progreso" aria-hidden="true"></span>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.aviso-global {
  --tono:#ee5c76; --fondo:#28171e;
  position:fixed;top:22px;right:22px;bottom:auto;z-index:120;
  display:flex;align-items:flex-start;gap:13px;
  width:min(420px,calc(100vw - 32px));padding:19px 42px 21px 17px;
  border:1px solid color-mix(in srgb,var(--tono) 48%,transparent);
  border-left:4px solid var(--tono);border-radius:15px;
  background:linear-gradient(125deg,var(--fondo),#16191e);
  box-shadow:0 18px 48px #0009,0 0 25px color-mix(in srgb,var(--tono) 9%,transparent);
  color:#f6f7f9;overflow:hidden;
}
.aviso-global--exito{--tono:#4ddd98;--fondo:#163328}
.aviso-global--aviso{--tono:#f3c66b;--fondo:#302819}
.aviso-global__icon{display:grid;place-items:center;flex:0 0 29px;height:29px;border-radius:50%;background:color-mix(in srgb,var(--tono) 16%,transparent);color:var(--tono);font-size:19px;font-weight:800}
.aviso-global__contenido{display:flex;flex-direction:column;gap:4px;min-width:0}
.aviso-global__contenido strong{font-size:14px;font-weight:800;line-height:1.3}
.aviso-global__nombre{font-size:13px;font-weight:650;color:#e0e4e9}
.aviso-global__contenido p{margin:2px 0 0;color:#b7bec8;font-size:12px;line-height:1.5;overflow-wrap:anywhere}
.aviso-global__cerrar{position:absolute;right:12px;top:10px;border:0;background:transparent;color:#9ca5b0;font-size:23px;cursor:pointer}
.aviso-global__cerrar:hover{color:white}
.aviso-global__progreso{position:absolute;bottom:0;left:0;height:3px;width:100%;background:var(--tono);opacity:.7;animation:aviso-tiempo 7.5s linear forwards}
.aviso-deslizar-enter-active,.aviso-deslizar-leave-active{transition:opacity .2s,transform .2s}
.aviso-deslizar-enter-from,.aviso-deslizar-leave-to{opacity:0;transform:translateY(-12px)}
@keyframes aviso-tiempo{to{width:0}}
@media(max-width:520px){.aviso-global{top:12px;right:12px;width:calc(100vw - 24px)}}
</style>
