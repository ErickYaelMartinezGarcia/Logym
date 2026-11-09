<script setup>
import AppShell from '../components/AppShell.vue'
import VisitaOcasional from '../components/VisitaOcasional.vue'
import { duracionPlan } from '../services/duracionPlanes'
import { useCheckIn } from '../composables/useCheckIn'

const {
  consultaManual, resultadosManual, buscandoManual, registrandoManual, mensajeManual,
  busquedaAbierta, modalManual, asistenciaManualRegistrada, lecturasPendientes,
  buscarManual, seleccionarManual, confirmarManual, cancelarBusqueda,
  tiposMembresia, tipoMembresiaId,
  router, mensaje, estadoCheckin, cargando, procesandoRenovacion, clienteDetectado, modalAbierta,
  confirmandoRenovacion, lectorActivo, estadoLector, formatearFecha, etiquetaEstado, diasRestantes,
  capturarAcceso, cerrarModal, solicitarRenovacion, renovar
} = useCheckIn()
</script>

<template>
  <AppShell>
    <div class="checkin-layout">
      <section class="checkin-copy">
        <p class="eyebrow">CONTROL DE ACCESO / 00</p>
        <h1 class="page-title">Listo para<br><span class="accent-text">entrenar?</span></h1>
        <p class="muted checkin-copy__description">Identifica tu huella y entra a VIRTUS sin filas, sin tarjetas y sin
          complicaciones.</p>
        <div class="neon-line checkin-copy__line"></div>
      </section>
      <section class="surface checkin-card">
        <form class="manual-search" role="search" aria-label="Buscar cliente para asistencia manual" @submit.prevent="buscarManual">
          <label for="buscar-manual">Acceso manual</label>
          <div class="manual-search__controls">
            <input id="buscar-manual" v-model="consultaManual" maxlength="120" placeholder="Nombre, teléfono o ID" :disabled="registrandoManual || modalAbierta" />
            <button class="secondary-button" :disabled="buscandoManual || registrandoManual || modalAbierta || !consultaManual.trim()">Buscar</button>
          </div>
        </form>
        <div v-if="busquedaAbierta" class="checkin-content manual-results-view">
          <div class="manual-results-heading">
            <h2>Seleccionar cliente</h2>
            <button type="button" class="text-button" @click="cancelarBusqueda">Volver al lector</button>
          </div>
          <p v-if="buscandoManual" role="status">Buscando clientes…</p>
          <p v-else class="muted manual-search-feedback" role="status">{{ mensajeManual }}</p>
          <ul class="manual-results">
            <li v-for="cliente in resultadosManual" :key="cliente.id">
              <button type="button" class="manual-result" @click="seleccionarManual(cliente)">
                <strong>{{ cliente.nombre }}</strong>
                <small>ID #{{ cliente.id }} · {{ cliente.telefono || 'Sin teléfono' }}</small>
                <small>{{ etiquetaEstado(cliente.estado_membresia) }}</small>
              </button>
            </li>
          </ul>
        </div>
        <div v-else class="checkin-content">
        <div class="fingerprint-orb" :class="{ 'fingerprint-orb--active': cargando || lectorActivo }">
          <div class="fingerprint-ring ring-one"></div>
          <div class="fingerprint-ring ring-two"></div><span>⌁</span>
        </div>
        <div class="reader-status"><span class="reader-status__dot"
            :class="{ 'reader-status__dot--active': lectorActivo }"></span>
            <span>{{ lectorActivo ? 'Lector Hamster Pro20 · listo' : estadoLector === 'no_disponible' ? 'Lector no disponible' : estadoLector === 'recuperando' ? 'Recuperando lector...' : 'Iniciando lector...' }}</span></div>
        <div class="checkin-status" :class="`checkin-status--${estadoCheckin}`">
          <p class="checkin-message">{{ mensaje }}</p>
        </div> <button class="text-button text-button--secondary" :disabled="cargando" @click="capturarAcceso(true)">{{
          cargando ? 'Leyendo dispositivo...' : 'Capturar huella' }}</button><button class="text-button"
          @click="router.push('/registro')">¿Nuevo miembro? Regístralo aquí</button>
        </div>
        <div v-if="modalAbierta && clienteDetectado" class="cliente-modal-backdrop" @click.self="cerrarModal">
          <div class="cliente-modal" :class="{
            'cliente-modal--valid': (!modalManual || asistenciaManualRegistrada) && ['activa', 'por_vencer'].includes(clienteDetectado.estado_membresia),
            'cliente-modal--expired': clienteDetectado.estado_membresia === 'vencida'
          }" role="dialog" aria-modal="true" aria-label="Información del cliente">
            <button class="modal-close" aria-label="Cerrar información del cliente" :disabled="registrandoManual" @click="cerrarModal">×</button>
            <div v-if="(!modalManual || asistenciaManualRegistrada) && ['activa', 'por_vencer'].includes(clienteDetectado.estado_membresia)" class="access-approved"><span>✓</span> ACCESO
              VÁLIDO</div>
            <p v-if="modalManual" class="eyebrow">{{ asistenciaManualRegistrada ? 'ASISTENCIA REGISTRADA' : 'CONFIRMACIÓN DEL OPERADOR' }}</p>
            <p v-if="!['success', 'warning'].includes(estadoCheckin)" class="checkin-status checkin-message" :class="`checkin-status--${estadoCheckin}`" role="status" aria-live="polite">{{ mensaje }}</p>
            <p v-else-if="estadoCheckin === 'success'" class="sr-only" role="status" aria-live="polite">{{ mensaje }}</p>
            <div class="panel-head">
              <div v-if="clienteDetectado.foto_base64" class="cliente-foto-frame">
                <img :src="clienteDetectado.foto_base64" alt="Foto del cliente" class="cliente-foto" />
              </div>
              <div v-else-if="modalManual" class="cliente-foto-frame cliente-avatar" aria-label="Sin fotografía">
                <svg viewBox="0 0 64 64" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><circle cx="32" cy="23" r="10"/><path d="M13 54c0-12 8-19 19-19s19 7 19 19"/></svg>
                <small>Sin fotografía</small>
              </div>
              <div class="cliente-details">
                <div class="cliente-info">
                  <strong>{{ clienteDetectado.nombre }}</strong>
                  <small>ID #{{ clienteDetectado.id }} · Cliente: {{ clienteDetectado.telefono || 'Sin teléfono'
                    }}</small>
                  <small>Emergencia: {{ clienteDetectado.telefono_emergencia || 'Sin teléfono' }}</small>
                  <small>Tipo de membresía: {{ clienteDetectado.tipo_membresia || 'Sin plan registrado' }}</small>
                  <span class="estado-chip" :class="`estado-${clienteDetectado.estado_membresia}`">{{
                    etiquetaEstado(clienteDetectado.estado_membresia) }}</span>
                </div>
                <div class="fechas-grid">
                  <div><small>Inicio</small>
                    <p>{{ formatearFecha(clienteDetectado.fecha_inicio_membresia) }}</p>
                  </div>
                  <div><small>Fin</small>
                    <p>{{ formatearFecha(clienteDetectado.fecha_fin_membresia) }}</p>
                  </div>
                </div>
                <p class="Dashboard-dias">{{ diasRestantes(clienteDetectado) }}</p>

                <div v-if="!modalManual && clienteDetectado.estado_membresia === 'vencida'" class="renovacion-box">
                  <p class="muted renewal-label">Membresía vencida</p>
                  <p class="renewal-copy">Puedes autorizar una nueva membresía con fecha base de hoy.</p>
                  <button v-if="!confirmandoRenovacion" class="secondary-button" @click="solicitarRenovacion">Renovar
                    membresía</button>
                  <div v-else class="renewal-confirmation">
                    <label>Plan para renovar (sin inscripción)
                      <select v-model="tipoMembresiaId" class="renewal-select">
                        <option v-for="plan in tiposMembresia" :key="plan.id" :value="String(plan.id)">{{ plan.nombre }} · {{ duracionPlan(plan) }} — {{ (plan.precio_centavos / 100).toLocaleString("es-MX", {style:"currency",currency:"MXN"}) }}</option>
                      </select>
                    </label>
                    <strong>¿Autorizar renovación?</strong>
                    <p>Se aplicará la duración del plan seleccionado, sin cobrar inscripción.</p>
                    <div class="renewal-actions">
                      <button class="text-button text-button--secondary" :disabled="procesandoRenovacion"
                        @click="confirmandoRenovacion = false">Cancelar</button>
                      <button class="secondary-button" :disabled="procesandoRenovacion" @click="renovar">{{
                        procesandoRenovacion ? 'Guardando...' : 'Aceptar renovación' }}</button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div v-if="modalManual && !asistenciaManualRegistrada" class="manual-actions">
              <div class="renewal-actions">
                <button type="button" class="text-button" :disabled="registrandoManual" @click="cerrarModal">Cancelar y volver al lector</button>
                <button type="button" class="secondary-button" :disabled="registrandoManual" @click="confirmarManual">{{ registrandoManual ? 'Registrando…' : 'Registrar asistencia' }}</button>
              </div>
            </div>
            <button v-else-if="modalManual" type="button" class="secondary-button manual-return" @click="cerrarModal">Volver al lector</button>
            <p v-if="lecturasPendientes" class="muted manual-queued" role="status">Hay {{ lecturasPendientes }} resultado(s) del lector pendiente(s). Se mostrarán al cerrar esta ventana.</p>
          </div>
        </div>
      </section>
    </div>
    <VisitaOcasional :bloqueada="modalAbierta" />
  </AppShell>
</template>

<style scoped>
.checkin-layout { padding-bottom:90px; }
.manual-search { text-align: left; padding-bottom: 20px; margin-bottom: 22px; border-bottom: 1px solid #343941; }
.manual-search label { color: #aeb3bc; font-size: 12px; }
.manual-search__controls { display: flex; gap: 8px; margin-top: 9px; }
.manual-search input { min-width: 0; width: 100%; padding: 11px 12px; color: #fff; background: #111317; border: 1px solid #343941; border-radius: 10px; }
.manual-search input:focus { outline: 2px solid #ee2046; outline-offset: 2px; }
.manual-search__controls .secondary-button { width: auto; }
.checkin-content { height: 340px; }
.manual-results-view { display: flex; flex-direction: column; text-align: left; }
.manual-results-heading { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
.manual-results-heading h2 { font-size: 15px; }
.manual-results-heading .text-button { margin: 0; color: #d3d7de; }
.manual-search-feedback { font-size: 12px; line-height: 1.6; margin: 14px 0; }
.manual-results { list-style: none; padding: 0; flex: 1; min-height: 0; overflow-y: auto; }
.manual-results li { margin: 8px 0; }
.manual-result { display: grid; gap: 5px; width: 100%; padding: 13px; text-align: left; color: #fff; background: #171a1f; border: 1px solid #343941; border-radius: 10px; }
.manual-result:hover, .manual-result:focus-visible { border-color: #ee2046; }
.manual-result small { color: #aeb3bc; font-size: 11px; }
.cliente-avatar { display: grid; place-content: center; justify-items: center; gap: 8px; color: #8f96a1; }
.cliente-avatar svg { width: 80px; height: 80px; }
.manual-actions { margin-top: 24px; padding-top: 20px; border-top: 1px solid #343941; }
.manual-queued { margin: 12px 0; font-size: 12px; line-height: 1.6; }
.manual-actions .renewal-actions { flex-wrap: wrap; }
.manual-actions .secondary-button { width: auto; }
.manual-return { margin-top: 22px; }
.checkin-layout {
  display: grid;
  grid-template-columns: 1fr minmax(320px, 430px);
  align-items: center;
  gap: clamp(20px, 5vw, 90px);
}

.checkin-card {
  padding: clamp(18px, 3vh, 36px) 32px;
  text-align: center;
}

.fingerprint-orb {
  position: relative;
  display: grid;
  place-items: center;
  width: clamp(115px, 21vh, 170px);
  height: clamp(115px, 21vh, 170px);
  margin: 0 auto 18px;
  border: 1px solid #5a1b2a;
  border-radius: 50%;
  color: #ee2046;
  background: radial-gradient(circle, rgba(238, 32, 70, .2), rgba(238, 32, 70, .02) 58%, transparent 60%);
  box-shadow: 0 0 60px rgba(238, 32, 70, .14);
}

.fingerprint-orb--active {
  animation: reader-pulse 2s ease-in-out infinite;
}

.fingerprint-orb span {
  font-size: clamp(48px, 9vh, 72px);
  line-height: 1;
  transform: rotate(90deg);
  text-shadow: 0 0 25px #ee2046;
}

.fingerprint-ring {
  position: absolute;
  border: 1px solid rgba(238, 32, 70, .27);
  border-radius: 50%;
}

.ring-one {
  inset: 14px;
}

.ring-two {
  inset: 29px;
}

.checkin-status {
  min-height: 58px;
  display: grid;
  place-items: center;
  margin: 0 0 12px;
  padding: 9px 14px;
  border: 1px solid #343941;
  border-radius: 12px;
  background: #171a1f;
  transition: .3s;
}

.checkin-status--processing {
  border-color: #3971c4;
  background: #14223a;
}

.checkin-status--success {
  border-color: #287c58;
  background: #103024;
}

.checkin-status--error {
  border-color: #8c2942;
  background: #351923;
}

.checkin-message {
  color: #c8cbd0;
  font-size: 14px;
  margin: 0;
}

.checkin-status--success .checkin-message {
  color: #8ce8b8;
}

.checkin-status--error .checkin-message {
  color: #ff9caf;
}

.reader-status {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  margin: -10px 0 14px;
  color: #8f96a1;
  font-size: 11px;
}

.reader-status__dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #737983;
}

.reader-status__dot--active {
  background: #44d483;
  box-shadow: 0 0 10px #44d483;
}

.text-button {
  margin-top: 19px;
  border: 0;
  color: #8e949e;
  background: transparent;
  font-size: 12px;
}

.text-button--secondary {
  margin-top: 0;
  color: #656c77;
  font-size: 11px;
}

.text-button:hover {
  color: #ee2046;
}

.cliente-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgba(0, 0, 0, .7);
  backdrop-filter: blur(7px);
}

.cliente-modal {
  position: relative;
  width: min(900px, 100%);
  max-height: calc(100vh - 48px);
  overflow-y: auto;
  padding: 34px;
  border: 1px solid #343941;
  border-radius: 20px;
  text-align: left;
  background: linear-gradient(145deg, #1c1f24, #111317);
  box-shadow: 0 24px 80px rgba(0, 0, 0, .55);
}

.cliente-modal--valid {
  border-color: #287c58;
  box-shadow: 0 0 38px rgba(68, 212, 131, .16), 0 24px 80px rgba(0, 0, 0, .55);
}

.cliente-modal--expired {
  border: 2px solid #b52b48;
  box-shadow: 0 0 38px rgba(238, 32, 70, .22), 0 24px 80px rgba(0, 0, 0, .55);
}

.modal-close {
  position: absolute;
  top: 14px;
  right: 17px;
  width: 34px;
  height: 34px;
  border: 1px solid #343941;
  border-radius: 50%;
  color: #aeb3bc;
  background: #171a1f;
  font-size: 22px;
  line-height: 1;
}

.modal-close:hover {
  color: #fff;
  border-color: #ee2046;
}

.access-approved {
  display: flex;
  align-items: center;
  gap: 7px;
  margin-bottom: 13px;
  color: #6de1a9;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: .12em;
}

.access-approved span {
  display: grid;
  place-items: center;
  width: 20px;
  height: 20px;
  border: 1px solid #44d483;
  border-radius: 50%;
}

.panel-head {
  display: grid;
  grid-template-columns: 300px minmax(0, 1fr);
  gap: 28px;
  align-items: center;
  padding-right: 32px;
}

.cliente-foto-frame {
  width: 300px;
  height: 300px;
  flex: none;
  overflow: hidden;
  border: 2px solid #4c5260;
  border-radius: 16px;
  background: #0e1013;
  box-shadow: 0 10px 28px rgba(0, 0, 0, .3);
}

.cliente-foto {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 0;
}

.cliente-details {
  min-width: 0;
}

.cliente-info strong {
  display: block;
  font-size: 24px;
  line-height: 1.15;
  letter-spacing: -.02em;
}

.cliente-info small {
  display: block;
  margin-top: 9px;
  color: #aeb3bc;
  font-size: 13px;
}

.estado-chip {
  display: inline-flex;
  margin-top: 15px;
  padding: 6px 11px;
  border-radius: 999px;
  border: 1px solid transparent;
  font-size: 11px;
  font-weight: 700;
}

.estado-vencida {
  color: #ff7a93;
  border-color: #6e1d30;
  background: #3c1721;
}

.estado-por_vencer {
  color: #f6d873;
  border-color: #60501b;
  background: #352b10;
}

.estado-activa {
  color: #6de1a9;
  border-color: #1b5e44;
  background: #103024;
}

.estado-sin_fecha {
  color: #9aa1ab;
  border-color: #414750;
  background: #22262d;
}

.fechas-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-top: 22px;
}

.fechas-grid small {
  color: #8f96a1;
  font-size: 11px;
}

.fechas-grid p {
  margin: 6px 0 0;
  font-size: 14px;
}

.Dashboard-dias {
  margin: 16px 0 0;
  font-size: 14px;
  color: #d3d7de;
}

.renovacion-box {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #2e333b;
}

.renewal-copy {
  margin: 7px 0 12px;
  color: #c8cbd0;
  font-size: 12px;
  line-height: 1.5;
}

.renewal-select { display:block; width:100%; padding:10px; margin:8px 0; color:white; background:#171a1f; border:1px solid #60501b; border-radius:8px; }

.renewal-confirmation {
  padding: 13px;
  border: 1px solid #60501b;
  border-radius: 10px;
  background: #211c0d;
}

.renewal-confirmation strong {
  font-size: 13px;
}

.renewal-confirmation p {
  margin: 6px 0 12px;
  color: #c8cbd0;
  font-size: 11px;
  line-height: 1.45;
}

.renewal-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
}

.renewal-actions .text-button {
  margin: 0;
}

.secondary-button {
  width: 100%;
  padding: 11px 12px;
  border: 1px solid #353a42;
  border-radius: 10px;
  color: #fff;
  background: #1a1d22;
  font-size: 12px;
  font-weight: 700;
}

.secondary-button:hover {
  border-color: #ee2046;
}

.secondary-button:disabled {
  opacity: .55;
  cursor: not-allowed;
}

@keyframes reader-pulse {
  50% {
    box-shadow: 0 0 75px rgba(68, 212, 131, .2);
    border-color: #287c58;
  }
}

@media(max-width:800px) {
  .checkin-layout {
    grid-template-columns: 1fr;
    min-height: auto;
    gap: 35px
  }

  .checkin-copy {
    text-align: center
  }

  .checkin-copy .neon-line {
    margin-left: auto;
    margin-right: auto
  }

  .checkin-card {
    padding: 30px 22px
  }

  .fechas-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr))
  }

  .cliente-modal {
    padding: 27px 20px;
  }

  .panel-head {
    grid-template-columns: 240px minmax(0, 1fr);
    gap: 16px;
    padding-right: 20px;
  }

  .cliente-foto-frame {
    width: 240px;
    height: 240px;
  }
}

@media(max-width:520px) {
  .panel-head {
    grid-template-columns: 1fr;
    padding-right: 0;
  }

  .cliente-foto-frame {
    width: min(280px, 100%);
    height: auto;
    aspect-ratio: 1;
    justify-self: center;
  }

  .fechas-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 8px;
  }
}
</style>
