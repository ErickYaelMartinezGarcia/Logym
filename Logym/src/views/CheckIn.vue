<script setup>
import AppShell from '../components/AppShell.vue'
import { useCheckIn } from '../composables/useCheckIn'

const {
  router, mensaje, estadoCheckin, cargando, procesandoRenovacion, clienteDetectado, modalAbierta,
  confirmandoRenovacion, lectorActivo, formatearFecha, etiquetaEstado, diasRestantes,
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
        <div class="fingerprint-orb" :class="{ 'fingerprint-orb--active': cargando || lectorActivo }">
          <div class="fingerprint-ring ring-one"></div>
          <div class="fingerprint-ring ring-two"></div><span>⌁</span>
        </div>
        <div class="reader-status"><span class="reader-status__dot"
            :class="{ 'reader-status__dot--active': lectorActivo }"></span>
            <span>{{ lectorActivo ? 'Lector Hamster Pro20 · listo' : 'Iniciando lector...' }}</span></div>
        <div class="checkin-status" :class="`checkin-status--${estadoCheckin}`">
          <p class="checkin-message">{{ mensaje }}</p>
        </div> <button class="text-button text-button--secondary" :disabled="cargando" @click="capturarAcceso(true)">{{
          cargando ? 'Leyendo dispositivo...' : 'Capturar huella' }}</button><button class="text-button"
          @click="router.push('/registro')">¿Nuevo miembro? Regístralo aquí</button>
        <div v-if="modalAbierta && clienteDetectado" class="cliente-modal-backdrop" @click.self="cerrarModal">
          <div class="cliente-modal" :class="{
            'cliente-modal--valid': clienteDetectado.estado_membresia === 'activa',
            'cliente-modal--expired': clienteDetectado.estado_membresia === 'vencida'
          }" role="dialog" aria-modal="true" aria-label="Información del cliente">
            <button class="modal-close" aria-label="Cerrar información del cliente" @click="cerrarModal">×</button>
            <div v-if="clienteDetectado.estado_membresia === 'activa'" class="access-approved"><span>✓</span> ACCESO
              VÁLIDO</div>
            <div class="panel-head">
              <div v-if="clienteDetectado.foto_base64" class="cliente-foto-frame">
                <img :src="clienteDetectado.foto_base64" alt="Foto del cliente" class="cliente-foto" />
              </div>
              <div class="cliente-details">
                <div class="cliente-info">
                  <strong>{{ clienteDetectado.nombre }}</strong>
                  <small>ID #{{ clienteDetectado.id }} · Cliente: {{ clienteDetectado.telefono || 'Sin teléfono'
                    }}</small>
                  <small>Emergencia: {{ clienteDetectado.telefono_emergencia || 'Sin teléfono' }}</small>
                  <span class="estado-chip" :class="`estado-${clienteDetectado.estado_membresia}`">{{
                    etiquetaEstado(clienteDetectado.estado_membresia) }}</span>
                </div>
                <div class="fechas-grid">
                  <div><small>Registro</small>
                    <p>{{ formatearFecha(clienteDetectado.fecha_registro) }}</p>
                  </div>
                  <div><small>Inicio</small>
                    <p>{{ formatearFecha(clienteDetectado.fecha_inicio_membresia) }}</p>
                  </div>
                  <div><small>Fin</small>
                    <p>{{ formatearFecha(clienteDetectado.fecha_fin_membresia) }}</p>
                  </div>
                </div>
                <p class="Dashboard-dias">{{ diasRestantes(clienteDetectado) }}</p>

                <div v-if="clienteDetectado.estado_membresia === 'vencida'" class="renovacion-box">
                  <p class="muted renewal-label">Membresía vencida</p>
                  <p class="renewal-copy">Puedes autorizar una nueva membresía mensual con fecha base de hoy.</p>
                  <button v-if="!confirmandoRenovacion" class="secondary-button" @click="solicitarRenovacion">Renovar
                    membresía</button>
                  <div v-else class="renewal-confirmation">
                    <strong>¿Autorizar renovación?</strong>
                    <p>Se registrarán 30 días a partir de hoy.</p>
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
          </div>
        </div>
      </section>
    </div>
  </AppShell>
</template>

<style scoped>
.checkin-layout {
  display: grid;
  grid-template-columns: 1fr minmax(320px, 430px);
  align-items: center;
  gap: 12%;
  min-height: calc(100vh - 160px);
}

.checkin-card {
  padding: 42px 38px;
  text-align: center;
}

.fingerprint-orb {
  position: relative;
  display: grid;
  place-items: center;
  width: 180px;
  height: 180px;
  margin: 0 auto 26px;
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
  font-size: 78px;
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
  width: min(680px, 100%);
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
  grid-template-columns: 190px minmax(0, 1fr);
  gap: 28px;
  align-items: center;
  padding-right: 32px;
}

.cliente-foto-frame {
  width: 190px;
  height: 190px;
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
  grid-template-columns: repeat(3, minmax(0, 1fr));
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
    grid-template-columns: 1fr
  }

  .cliente-modal {
    padding: 27px 20px;
  }

  .panel-head {
    grid-template-columns: 130px minmax(0, 1fr);
    gap: 16px;
    padding-right: 20px;
  }

  .cliente-foto-frame {
    width: 130px;
    height: 130px;
  }
}

@media(max-width:520px) {
  .panel-head {
    grid-template-columns: 1fr;
    padding-right: 0;
  }

  .cliente-foto-frame {
    width: 170px;
    height: 170px;
  }

  .fechas-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
  }
}
</style>
