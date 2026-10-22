<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { useRouter } from 'vue-router'
import { inicializarBaseDeDatos } from '../db'

const router = useRouter()
const mensaje = ref('Coloca tu huella')
const cargando = ref(false)

/**
 * Inicializa la base de datos cuando abre la pantalla de check-in.
 */
onMounted(async () => {
  try {
    await inicializarBaseDeDatos()
  } catch (error) {
    mensaje.value = 'No se pudo abrir gym.db'
  }
})

/**
 * Captura una huella usando el comando de Tauri para validar acceso.
 */
async function capturarHuella() {
  try {
    cargando.value = true
    const template = await invoke('capturar_huella')
    mensaje.value = `Huella capturada: ${template.slice(0, 18)}...`
  } catch (error) {
    mensaje.value = 'Error al capturar huella'
  } finally {
    cargando.value = false
  }
}

/**
 * Abre la pantalla de registro para dar de alta clientes.
 */
function irARegistro() {
  router.push('/registro')
}

/**
 * Abre el dashboard para revisar métricas del gimnasio.
 */
function irADashboard() {
  router.push('/dashboard')
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center px-6">
    <div class="w-full max-w-xl bg-[#141414] border border-[#1F1F1F] rounded-[24px] shadow-2xl backdrop-blur-xl p-10 text-center space-y-8">
      <h1 class="text-3xl font-semibold text-[#C41E3A]">GYM OS</h1>
      <div class="mx-auto w-40 h-40 rounded-full bg-[#8B0000] animate-pulse flex items-center justify-center shadow-2xl">
        <span class="text-white font-semibold">Huella</span>
      </div>
      <p class="text-lg text-gray-200">{{ mensaje }}</p>
      <button
        class="w-full py-4 rounded-2xl bg-[#8B0000] hover:bg-[#A00000] transition font-semibold disabled:opacity-50"
        @click="capturarHuella"
        :disabled="cargando"
      >
        {{ cargando ? 'Capturando...' : 'Capturar huella' }}
      </button>
      <div class="grid grid-cols-2 gap-4">
        <button class="py-3 rounded-2xl bg-[#1C1C1C] border border-[#1F1F1F] hover:bg-[#2A2A2A]" @click="irARegistro">Ir a Registro</button>
        <button class="py-3 rounded-2xl bg-[#1C1C1C] border border-[#1F1F1F] hover:bg-[#2A2A2A]" @click="irADashboard">Ir a Dashboard</button>
      </div>
    </div>
  </div>
</template>
