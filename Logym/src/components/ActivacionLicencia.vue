<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
const emit = defineEmits(['activada'])
const solicitud = ref('')
const licencia = ref('')
const error = ref('')
const esperando = ref(false)
onMounted(async () => {
  try { solicitud.value = await invoke('solicitud_licencia') }
  catch (e) { error.value = String(e) }
})
async function activar() {
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
      <h1 class="text-2xl font-bold">Activación de Virtus Gym Sistem</h1>
      <p class="text-slate-300 text-sm">Licencia permanente sin conexión a internet. Envía este código al creador para recibir tu licencia.</p>
      <label class="block text-sm" for="solicitud">Código de esta PC</label>
      <textarea id="solicitud" class="w-full bg-black border border-slate-600 rounded-lg p-3 text-sm" rows="3" readonly :value="solicitud" />
      <label class="block text-sm" for="licencia">Licencia firmada</label>
      <textarea id="licencia" v-model="licencia" class="w-full bg-black border border-slate-600 rounded-lg p-3 text-sm" rows="4" placeholder="Pega aquí tu licencia" />
      <p v-if="error" class="text-red-400 text-sm" role="alert">{{ error }}</p>
      <button class="bg-amber-400 text-black font-bold px-6 py-3 rounded-lg disabled:opacity-50" :disabled="esperando || !licencia.trim()" @click="activar">Activar licencia</button>
      <p class="text-xs text-slate-400">Versión 0.1.0 · Erick Yael Martinez Garcia · Soporte pendiente</p>
    </section>
  </main>
</template>
