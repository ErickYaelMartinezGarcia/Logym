<script setup>
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import AppShell from '../components/AppShell.vue'
import { inicializarBaseDeDatos, guardarClienteEnBaseDeDatos } from '../db'

const nombre = ref('')
const telefono = ref('')
const fechaRegistro = ref('')
const fechaInicioMembresia = ref('')
const fechaFinMembresia = ref('')
const template1 = ref('')
const template2 = ref('')
const fotoBase64 = ref('')
const videoRef = ref(null)
const canvasRef = ref(null)
const stream = ref(null)
const camaraActiva = ref(false)
const camaraDisponible = ref(true)
const mensaje = ref('Completa los datos y captura la biometría')
const tipoMensaje = ref('neutral')

function mostrarMensaje(texto, tipo = 'neutral') {
  mensaje.value = texto
  tipoMensaje.value = tipo
}

function sumarDias(fechaISO, dias) {
  const fecha = new Date(`${fechaISO}T00:00:00`)
  fecha.setDate(fecha.getDate() + dias)
  return fecha.toISOString().slice(0, 10)
}

async function iniciarCamara() {
  if (!navigator.mediaDevices?.getUserMedia) {
    camaraDisponible.value = false
    mostrarMensaje('Este entorno no expone la cámara. Usa localhost o la aplicación Tauri.', 'error')
    return
  }
  try {
    stream.value = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'user', width: { ideal: 1280 } }, audio: false })
    videoRef.value.srcObject = stream.value
    camaraActiva.value = true
    mostrarMensaje('Cámara lista. Mira al frente y captura tu foto.', 'success')
  } catch (error) {
    const razon = error.name === 'NotAllowedError' ? 'Permiso de cámara rechazado.' : error.name === 'NotFoundError' ? 'No se encontró una cámara conectada.' : 'No fue posible abrir la cámara.'
    mostrarMensaje(`${razon} Revisa permisos del navegador o del sistema.`, 'error')
  }
}

function detenerCamara() {
  stream.value?.getTracks().forEach((track) => track.stop())
  stream.value = null
  camaraActiva.value = false
}

function capturarFoto() {
  const video = videoRef.value
  const canvas = canvasRef.value
  if (!camaraActiva.value || !video?.videoWidth || !canvas) {
    mostrarMensaje('Primero activa la cámara y espera a que aparezca la imagen.', 'error')
    return
  }
  canvas.width = video.videoWidth
  canvas.height = video.videoHeight
  canvas.getContext('2d').drawImage(video, 0, 0, canvas.width, canvas.height)
  fotoBase64.value = canvas.toDataURL('image/jpeg', .86)
  mostrarMensaje('Foto capturada correctamente.', 'success')
}

async function capturarHuella(numero) {
  try {
    const template = await invoke('capturar_huella')
    if (numero === 1) template1.value = template
    else template2.value = template
    mostrarMensaje(`Huella ${numero} capturada. Repite para confirmar la identidad.`, 'success')
  } catch (error) {
    console.error(`Error al capturar huella ${numero}:`, error)
    mostrarMensaje('El lector no está conectado o aún usa el modo demostración.', 'error')
  }
}

async function guardarCliente() {
  if (!nombre.value.trim() || !template1.value || !template2.value) {
    mostrarMensaje('Completa el nombre y las dos capturas de huella.', 'error')
    return
  }
  if (!fechaRegistro.value || !fechaInicioMembresia.value || !fechaFinMembresia.value) {
    mostrarMensaje('Captura las fechas de registro, inicio y fin de membresía.', 'error')
    return
  }
  if (fechaFinMembresia.value < fechaInicioMembresia.value) {
    mostrarMensaje('La fecha de fin no puede ser anterior al inicio de membresía.', 'error')
    return
  }
  try {
    const clienteId = await guardarClienteEnBaseDeDatos({
      nombre: nombre.value.trim(),
      telefono: telefono.value.trim(),
      fechaRegistro: fechaRegistro.value,
      fechaInicioMembresia: fechaInicioMembresia.value,
      fechaFinMembresia: fechaFinMembresia.value,
      fotoBase64: fotoBase64.value,
      templateHuella: template1.value
    })
    await invoke('guardar_template', { clienteId: String(clienteId), template: template1.value })
    nombre.value = ''; telefono.value = ''; template1.value = ''; template2.value = ''; fotoBase64.value = ''
    fechaRegistro.value = new Date().toISOString().slice(0, 10)
    fechaInicioMembresia.value = fechaRegistro.value
    fechaFinMembresia.value = sumarDias(fechaInicioMembresia.value, 30)
    mostrarMensaje('Miembro registrado. Ya puede utilizar el check-in.', 'success')
  } catch (error) {
    console.error('Error al guardar miembro:', error)
    mostrarMensaje('No se pudo guardar el miembro. Revisa la base de datos.', 'error')
  }
}

onMounted(async () => {
  const hoy = new Date().toISOString().slice(0, 10)
  fechaRegistro.value = hoy
  fechaInicioMembresia.value = hoy
  fechaFinMembresia.value = sumarDias(hoy, 30)
  try { await inicializarBaseDeDatos() } catch (error) { mostrarMensaje('No se pudo inicializar la base de datos local.', 'error') }
})
onBeforeUnmount(detenerCamara)
</script>

<template>
  <AppShell>
    <div style="margin-bottom:30px"><p class="eyebrow">NUEVO MIEMBRO / 02</p><h1 class="page-title">Hazlo personal.</h1><p class="muted" style="margin-top:12px">Registra una identidad completa para que el acceso sea rápido y seguro.</p></div>
    <div class="registration-grid" style="display:grid; grid-template-columns:minmax(0,1fr) minmax(300px,.8fr); gap:18px">
      <section class="surface" style="padding:26px">
        <p class="eyebrow">DATOS BÁSICOS</p><h2 style="margin:7px 0 22px; font-size:21px">Información del miembro</h2>
        <label class="field-label">Nombre completo<input v-model="nombre" type="text" placeholder="Ej. Alex Rivera" /></label>
        <label class="field-label">Teléfono <span class="muted">(opcional)</span><input v-model="telefono" type="tel" placeholder="+52 55 0000 0000" /></label>
        <div class="date-grid">
          <label class="field-label">Fecha de registro<input v-model="fechaRegistro" type="date" /></label>
          <label class="field-label">Inicio de membresía<input v-model="fechaInicioMembresia" type="date" /></label>
          <label class="field-label">Fin de membresía<input v-model="fechaFinMembresia" type="date" /></label>
        </div>
        <div class="capture-row"><button class="capture-card" :class="{ done: template1 }" @click="capturarHuella(1)"><span class="capture-icon">01</span><span><strong>{{ template1 ? 'Huella guardada' : 'Primera huella' }}</strong><small>{{ template1 ? 'Captura completa' : 'Coloca el dedo en el lector' }}</small></span><b>›</b></button><button class="capture-card" :class="{ done: template2 }" @click="capturarHuella(2)"><span class="capture-icon">02</span><span><strong>{{ template2 ? 'Huella confirmada' : 'Confirmación' }}</strong><small>{{ template2 ? 'Captura completa' : 'Repite la misma huella' }}</small></span><b>›</b></button></div>
        <button class="primary-button" @click="guardarCliente">Guardar miembro <span>→</span></button>
        <p class="feedback" :class="`feedback--${tipoMensaje}`">{{ mensaje }}</p>
      </section>
      <section class="surface" style="padding:26px">
        <div style="display:flex; justify-content:space-between; align-items:start"><div><p class="eyebrow">IDENTIDAD VISUAL</p><h2 style="margin:7px 0 4px; font-size:21px">Foto de perfil</h2></div><span class="camera-status" :class="{ active: camaraActiva }"><i></i>{{ camaraActiva ? 'EN VIVO' : 'INACTIVA' }}</span></div>
        <div class="camera-frame"><video ref="videoRef" autoplay playsinline></video><div v-if="!camaraActiva" class="camera-placeholder"><span>◎</span><small>{{ camaraDisponible ? 'Activa la cámara para comenzar' : 'Cámara no disponible en este entorno' }}</small></div></div>
        <div class="camera-actions"><button class="secondary-button" @click="camaraActiva ? detenerCamara() : iniciarCamara()">{{ camaraActiva ? 'Apagar cámara' : 'Activar cámara' }}</button><button class="primary-button compact" :disabled="!camaraActiva" @click="capturarFoto">Tomar foto</button></div>
        <canvas ref="canvasRef" class="hidden"></canvas><img v-if="fotoBase64" :src="fotoBase64" alt="Foto del nuevo miembro" class="photo-preview" />
        <p class="muted" style="font-size:11px; line-height:1.5; margin:15px 0 0">La cámara solo se usa al pulsar “Activar cámara”. La foto se guarda localmente junto al perfil.</p>
      </section>
    </div>
  </AppShell>
</template>

<style scoped>
.field-label { display:block; color:#9da3ad; font-size:12px; margin-bottom:16px; }
.field-label input { display:block; width:100%; margin-top:8px; padding:13px 14px; border:1px solid #343941; border-radius:11px; outline:0; color:#fff; background:#111317; transition:.2s; }
.field-label input:focus { border-color:#ee2046; box-shadow:0 0 0 3px rgba(238,32,70,.12); }
.date-grid { display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:10px; margin-bottom:6px; }
.capture-row { display:grid; grid-template-columns:1fr 1fr; gap:10px; margin:25px 0 17px; }
.capture-card { display:flex; align-items:center; gap:10px; min-width:0; padding:13px 11px; border:1px solid #343941; border-radius:12px; color:#fff; text-align:left; background:#16191e; transition:.2s; }
.capture-card:hover, .capture-card.done { border-color:#ee2046; }
.capture-card b { margin-left:auto; color:#ee2046; font-size:20px; }
.capture-card strong, .capture-card small { display:block; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.capture-card strong { font-size:12px; }.capture-card small { margin-top:4px; color:#797f89; font-size:10px; }
.capture-icon { display:grid; place-items:center; width:29px; height:29px; flex:none; border-radius:8px; color:#ee2046; background:#2b131b; font-size:10px; font-weight:800; }
.primary-button, .secondary-button { width:100%; padding:13px 16px; border:1px solid transparent; border-radius:11px; color:#fff; font-size:13px; font-weight:700; transition:.2s; }.primary-button { background:#ee2046; box-shadow:0 7px 20px rgba(238,32,70,.18); }.primary-button:hover { background:#ff3157; transform:translateY(-1px); }.primary-button:disabled { opacity:.4; cursor:not-allowed; transform:none; }.primary-button span { float:right; font-size:18px; line-height:12px; }.compact { width:auto; white-space:nowrap; }.secondary-button { border-color:#353a42; background:#1a1d22; }.secondary-button:hover { border-color:#ee2046; }
.feedback { min-height:18px; margin:14px 0 0; font-size:12px; }.feedback--error { color:#ff6d86; }.feedback--success { color:#55d996; }.feedback--neutral { color:#9298a2; }
.camera-status { color:#858b95; font-size:9px; font-weight:800; letter-spacing:.1em; }.camera-status i { display:inline-block; width:6px; height:6px; margin-right:6px; border-radius:50%; background:#777; }.camera-status.active { color:#55d996; }.camera-status.active i { background:#55d996; box-shadow:0 0 9px #55d996; }
.camera-frame { position:relative; display:grid; place-items:center; min-height:270px; margin-top:20px; overflow:hidden; border:1px solid #343941; border-radius:14px; background:#0c0e11; }.camera-frame video { display:block; width:100%; max-height:320px; object-fit:cover; transform:scaleX(-1); }.camera-placeholder { display:grid; place-items:center; gap:9px; color:#757b85; text-align:center; }.camera-placeholder span { color:#ee2046; font-size:52px; line-height:1; }.camera-placeholder small { font-size:11px; }.camera-actions { display:grid; grid-template-columns:1fr auto; gap:9px; margin-top:12px; }.photo-preview { width:100%; margin-top:14px; border-radius:12px; border:1px solid #ee2046; }
@media (max-width:800px) { .registration-grid { grid-template-columns:1fr !important; }.capture-row { grid-template-columns:1fr; }.date-grid { grid-template-columns:1fr; } }
</style>
