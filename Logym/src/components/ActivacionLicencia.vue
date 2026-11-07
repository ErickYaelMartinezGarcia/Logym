<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
const props = defineProps({ activa: Boolean, errorInicial: { type: String, default: '' } })
const emit = defineEmits(['activada', 'cerrar'])
const solicitud = ref('')
const codigo = ref(null)
const licencia = ref('')
const error = ref('')
const aviso = ref('')
const esperando = ref(false)
const cargandoCodigo = ref(true)
onMounted(async () => {
  try { solicitud.value = await invoke('solicitud_licencia') }
  catch (e) { error.value = 'No se pudo obtener el código de hardware. ' + String(e) }
  finally { cargandoCodigo.value = false }
})
async function copiar() {
  if (!solicitud.value) return
  aviso.value = ''
  try {
    try { await navigator.clipboard.writeText(solicitud.value) }
    catch {
      codigo.value.focus()
      codigo.value.select()
      if (!document.execCommand('copy')) throw Error('Selecciona el código y cópialo manualmente.')
    }
    aviso.value = 'Código copiado.'
  } catch (e) { aviso.value = 'No se pudo copiar. ' + String(e) }
}
async function activar() {
  if (esperando.value || !solicitud.value || !licencia.value.trim()) return
  esperando.value = true
  error.value = ''
  try {
    await invoke('activar_licencia', { licenciaTexto: licencia.value.trim() })
    emit('activada')
  } catch (e) { error.value = String(e) }
  finally { esperando.value = false }
}
</script>
<template>
  <main class="min-h-screen bg-[#090a0c] text-white flex items-center justify-center p-6">
    <section class="w-full max-w-xl rounded-2xl border border-slate-700 bg-[#151b22] p-8 space-y-5">
      <h1 class="text-2xl font-bold">{{ props.activa ? 'Licencia de esta computadora' : 'Activación de Logym' }}</h1>
      <p v-if="props.activa" role="status">Licencia válida. Activación permanente y offline vinculada al hardware.</p>
      <p v-else class="text-slate-300 text-sm">Envía el código de esta computadora al emisor y pega aquí la licencia firmada que recibas.</p>
      <label class="block text-sm" for="solicitud">Código de esta PC</label>
      <textarea id="solicitud" ref="codigo" class="w-full bg-black border border-slate-600 rounded-lg p-3 text-sm" rows="3" readonly :value="solicitud" />
      <p v-if="cargandoCodigo" role="status">Obteniendo código de hardware...</p>
      <button type="button" :disabled="!solicitud" @click="copiar">Copiar código de hardware</button>
      <p v-if="aviso" role="status">{{ aviso }}</p>
      <form v-if="!props.activa" @submit.prevent="activar">
        <label class="block text-sm" for="licencia">Licencia firmada</label>
        <textarea id="licencia" v-model="licencia" class="w-full bg-black border border-slate-600 rounded-lg p-3 text-sm" rows="4" placeholder="Pega aquí tu licencia" />
        <button type="submit" class="bg-amber-400 text-black font-bold px-6 py-3 rounded-lg disabled:opacity-50" :disabled="esperando || !solicitud || !licencia.trim()">{{ esperando ? 'Activando...' : 'Activar licencia' }}</button>
      </form>
      <p v-if="error || props.errorInicial" class="text-red-400 text-sm" role="alert">{{ error || props.errorInicial }}</p>
      <button v-if="props.activa" type="button" @click="emit('cerrar')">Volver al sistema</button>
    </section>
  </main>
</template>
