<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { inicializarBaseDeDatos, guardarClienteEnBaseDeDatos } from '../db'

const nombre = ref('')
const telefono = ref('')
const template1 = ref('')
const template2 = ref('')
const fotoBase64 = ref('')
const videoRef = ref(null)
const canvasRef = ref(null)
const mensaje = ref('Completa los datos del cliente')

/**
 * Inicia la cámara y la base de datos al entrar al registro.
 */
onMounted(async () => {
  try {
    await inicializarBaseDeDatos()
    const stream = await navigator.mediaDevices.getUserMedia({ video: true })
    if (videoRef.value) {
      videoRef.value.srcObject = stream
    }
  } catch (error) {
    mensaje.value = 'No se pudo abrir la cámara o la base de datos'
  }
})

/**
 * Toma una foto desde la webcam y la guarda en base64.
 */
function capturarFoto() {
  try {
    const video = videoRef.value
    const canvas = canvasRef.value
    if (!video || !canvas) return

    canvas.width = video.videoWidth
    canvas.height = video.videoHeight
    const contexto = canvas.getContext('2d')
    contexto.drawImage(video, 0, 0, canvas.width, canvas.height)
    fotoBase64.value = canvas.toDataURL('image/png')
    mensaje.value = 'Foto capturada correctamente'
  } catch (error) {
    mensaje.value = 'Error al capturar la foto'
  }
}

/**
 * Captura la primera huella del cliente para el registro.
 */
async function capturarHuella1() {
  try {
    template1.value = await invoke('capturar_huella')
    mensaje.value = 'Huella 1 capturada'
  } catch (error) {
    mensaje.value = 'Error al capturar huella 1'
  }
}

/**
 * Captura la segunda huella para confirmar el registro.
 */
async function capturarHuella2() {
  try {
    template2.value = await invoke('capturar_huella')
    mensaje.value = 'Huella 2 capturada'
  } catch (error) {
    mensaje.value = 'Error al capturar huella 2'
  }
}

/**
 * Guarda el cliente en SQLite y envía el template al backend de Tauri.
 */
async function guardarCliente() {
  try {
    if (!nombre.value || !template1.value || !template2.value) {
      mensaje.value = 'Faltan datos para guardar cliente'
      return
    }

    const templateFinal = template1.value

    const clienteId = await guardarClienteEnBaseDeDatos({
      nombre: nombre.value,
      telefono: telefono.value,
      fotoBase64: fotoBase64.value,
      templateHuella: templateFinal
    })

    await invoke('guardar_template', {
      clienteId: String(clienteId),
      template: templateFinal
    })

    mensaje.value = 'Cliente guardado correctamente'
    nombre.value = ''
    telefono.value = ''
    template1.value = ''
    template2.value = ''
  } catch (error) {
    mensaje.value = 'Error al guardar cliente'
  }
}
</script>

<template>
  <div class="min-h-screen px-6 py-10">
    <div class="max-w-5xl mx-auto bg-[#141414] border border-[#1F1F1F] rounded-[24px] shadow-2xl backdrop-blur-xl p-8">
      <h1 class="text-3xl font-semibold text-[#C41E3A] mb-8">Registro de Clientes</h1>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div class="space-y-4">
          <input v-model="nombre" type="text" placeholder="Nombre completo" class="w-full bg-[#0F0F0F] border border-[#1F1F1F] rounded-2xl p-4 outline-none focus:border-[#8B0000]" />
          <input v-model="telefono" type="text" placeholder="Teléfono" class="w-full bg-[#0F0F0F] border border-[#1F1F1F] rounded-2xl p-4 outline-none focus:border-[#8B0000]" />

          <div class="grid grid-cols-2 gap-4">
            <button class="py-3 rounded-2xl bg-[#8B0000] hover:bg-[#A00000]" @click="capturarHuella1">Capturar huella 1</button>
            <button class="py-3 rounded-2xl bg-[#8B0000] hover:bg-[#A00000]" @click="capturarHuella2">Capturar huella 2</button>
          </div>

          <button class="w-full py-3 rounded-2xl bg-[#8B0000] hover:bg-[#A00000] font-semibold" @click="guardarCliente">Guardar cliente</button>
          <p class="text-gray-300">{{ mensaje }}</p>
        </div>

        <div class="space-y-4">
          <div class="bg-[#0F0F0F] border border-[#1F1F1F] rounded-2xl p-4">
            <video ref="videoRef" autoplay playsinline class="w-full rounded-xl"></video>
          </div>
          <button class="w-full py-3 rounded-2xl bg-[#8B0000] hover:bg-[#A00000]" @click="capturarFoto">Capturar foto</button>
          <canvas ref="canvasRef" class="hidden"></canvas>
          <img v-if="fotoBase64" :src="fotoBase64" alt="Foto capturada" class="w-full rounded-2xl border border-[#1F1F1F]" />
        </div>
      </div>
    </div>
  </div>
</template>
