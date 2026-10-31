<script setup>
import AppShell from '../components/AppShell.vue'
import { useRegistroCliente } from '../composables/useRegistroCliente'

const registro = useRegistroCliente()
const {
  nombre, telefono, fechaRegistro, fechaInicioMembresia, fechaFinMembresia,
  telefonoEmergencia,
  template1, template2, mensaje, tipoMensaje, videoRef, canvasRef, fotoBase64,
  camaraActiva, camaraDisponible, iniciarCamara, detenerCamara, capturarFoto, repetirFoto,
  capturarHuellaCliente, guardar: guardarCliente
} = registro
</script>

<template>
  <AppShell>
    <div class="page-intro">
      <h1 class="page-title">Hazlo personal.</h1>
      <p class="muted page-intro__description">Registra una identidad completa para que el acceso sea rápido y seguro.</p></div>
    <div class="registration-grid">
      <section class="surface form-panel">
        <p class="eyebrow">DATOS BÁSICOS</p><h2 class="form-panel__title">Información del miembro</h2>
        <label class="field-label">Nombre completo<input v-model="nombre" type="text" placeholder="Ej. Alex Rivera" /></label>
        <div class="phone-grid">
          <label class="field-label">Teléfono del cliente <span>(opcional)</span><input v-model="telefono" type="tel" placeholder="+52 55 0000 0000" /></label>
          <label class="field-label">Teléfono de emergencia <span>(opcional)</span><input v-model="telefonoEmergencia" type="tel" placeholder="+52 55 0000 0000" /></label>
        </div>
        <div class="date-grid">
          <label class="field-label">Fecha de registro<input v-model="fechaRegistro" type="date" /></label>
          <label class="field-label">Inicio de membresía<input v-model="fechaInicioMembresia" type="date" /></label>
          <label class="field-label">Fin de membresía<input v-model="fechaFinMembresia" type="date" /></label>
        </div>
        <div class="capture-row"><button class="capture-card" :class="{ done: template1 }" @click="capturarHuellaCliente(1)"><span class="capture-icon">01</span><span><strong>{{ template1 ? 'Huella guardada' : 'Primera huella' }}</strong><small>{{ template1 ? 'Captura completa' : 'Coloca el dedo en el lector' }}</small></span><b>›</b></button><button class="capture-card" :class="{ done: template2 }" @click="capturarHuellaCliente(2)"><span class="capture-icon">02</span><span><strong>{{ template2 ? 'Huella confirmada' : 'Confirmación' }}</strong><small>{{ template2 ? 'Captura completa' : 'Repite la misma huella' }}</small></span><b>›</b></button></div>
        <button class="primary-button" @click="guardarCliente">Guardar miembro <span>→</span></button>
        <p class="feedback" :class="`feedback--${tipoMensaje}`">{{ mensaje }}</p>
      </section>
      <section class="surface form-panel">
        <div class="camera-panel__header"><div><p class="eyebrow">IDENTIDAD VISUAL</p><h2 class="camera-panel__title">Foto de perfil</h2></div><span class="camera-status" :class="{ active: camaraActiva }"><i></i>{{ camaraActiva ? 'EN VIVO' : 'INACTIVA' }}</span></div>
        <div class="camera-frame"><img v-if="fotoBase64" :src="fotoBase64" alt="Foto capturada del nuevo miembro" class="photo-preview" /><video v-show="!fotoBase64" ref="videoRef" autoplay playsinline></video><div v-if="!camaraActiva && !fotoBase64" class="camera-placeholder"><span>◎</span><small>{{ camaraDisponible ? 'Activa la cámara para comenzar' : 'Cámara no disponible en este entorno' }}</small></div></div>
        <div v-if="fotoBase64" class="camera-actions"><button class="secondary-button" @click="repetirFoto">Tomar otra foto</button></div><div v-else class="camera-actions"><button class="secondary-button" @click="camaraActiva ? detenerCamara() : iniciarCamara()">{{ camaraActiva ? 'Apagar cámara' : 'Activar cámara' }}</button><button class="primary-button compact" :disabled="!camaraActiva" @click="capturarFoto">Tomar foto</button></div>
        <canvas ref="canvasRef" class="hidden"></canvas>
        <p class="muted camera-help">La cámara solo se usa al pulsar “Activar cámara”. La foto se guarda localmente junto al perfil.</p>
      </section>
    </div>
  </AppShell>
</template>

<style scoped>
.field-label { display:block; color:#9da3ad; font-size:12px; margin-bottom:16px; }
.field-label span { color:#6f7680; font-size:10px; }
.phone-grid { display:grid; grid-template-columns:1fr 1fr; gap:10px; }
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
.camera-frame { position:relative; display:grid; place-items:center; min-height:270px; margin-top:20px; overflow:hidden; border:1px solid #343941; border-radius:14px; background:#0c0e11; }.camera-frame video { display:block; width:100%; max-height:320px; object-fit:cover; transform:scaleX(-1); }.camera-placeholder { display:grid; place-items:center; gap:9px; color:#757b85; text-align:center; }.camera-placeholder span { color:#ee2046; font-size:52px; line-height:1; }.camera-placeholder small { font-size:11px; }.camera-actions { display:grid; grid-template-columns:1fr auto; gap:9px; margin-top:12px; }.photo-preview { display:block; width:100%; height:320px; object-fit:cover; border-radius:12px; border:1px solid #ee2046; }
@media (max-width:800px) { .registration-grid { grid-template-columns:1fr !important; }.capture-row { grid-template-columns:1fr; }.date-grid, .phone-grid { grid-template-columns:1fr; } }
</style>
