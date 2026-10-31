<script setup>
import { ref, onMounted } from 'vue'
import AppShell from '../components/AppShell.vue'
import { adminConfigurado, crearPasswordAdmin, validarPasswordAdmin, listarPlanesAdmin, obtenerPrecioInscripcion, guardarPlanAdmin, guardarInscripcionAdmin } from '../services/tauriApi'

const configurado = ref(true)
const autorizado = ref(false)
const password = ref('')
const confirmar = ref('')
const mensaje = ref('')
const planes = ref([])
const inscripcion = ref(0)
const nuevo = ref({ nombre:'', meses:1, precio_pesos:0, activo:true })
const moneda = n => (n/100).toLocaleString('es-MX',{style:'currency',currency:'MXN'})
const aCentavos = n => Math.round(Number(n)*100)
async function cargar() {
  planes.value = (await listarPlanesAdmin(password.value)).map(p => ({...p, precio_pesos:p.precio_centavos/100}))
  inscripcion.value = (await obtenerPrecioInscripcion())/100
}
async function entrar() {
  mensaje.value=''
  try {
    if (!configurado.value) {
      if (password.value !== confirmar.value) throw Error('Las contraseñas no coinciden')
      await crearPasswordAdmin(password.value)
      configurado.value=true
    } else await validarPasswordAdmin(password.value)
    autorizado.value=true
    await cargar()
  } catch(e) { mensaje.value=String(e) }
}
async function guardarPlan(plan) {
  mensaje.value=''
  try {
    const { id, nombre, duracion_meses, meses, precio_pesos, activo } = plan
    await guardarPlanAdmin(password.value,{ id:id||null, nombre, meses:Number(duracion_meses||meses), precioCentavos:aCentavos(precio_pesos), activo })
    await cargar()
    mensaje.value='Plan guardado. Los cobros anteriores permanecen intactos.'
  } catch(e) { mensaje.value=String(e) }
}
async function crearPlan() {
  await guardarPlan(nuevo.value)
}
async function guardarInscripcion() {
  try {
    await guardarInscripcionAdmin(password.value,aCentavos(inscripcion.value))
    mensaje.value='Precio de inscripción actualizado para los próximos registros.'
  } catch(e) { mensaje.value=String(e) }
}
onMounted(async () => {
  try { configurado.value=await adminConfigurado() }
  catch(e) { mensaje.value=String(e) }
})
</script>
<template>
  <AppShell>
    <div class="page-intro"><h1 class="page-title">Administración de precios</h1><p class="muted">Cambios aplicables únicamente a nuevos cobros. Los pagos históricos no cambian.</p></div>
    <section v-if="!autorizado" class="surface admin-panel">
      <h2>{{ configurado?'Contraseña administrativa':'Crear contraseña administrativa' }}</h2>
      <p class="muted">{{ configurado?'Acceso restringido':'Primera configuración: mínimo 12 caracteres. Guarda esta contraseña en un lugar seguro.' }}</p>
      <input v-model="password" type="password" autocomplete="off" placeholder="Contraseña" />
      <input v-if="!configurado" v-model="confirmar" type="password" autocomplete="off" placeholder="Confirmar contraseña" />
      <button @click="entrar">{{ configurado?'Entrar':'Crear contraseña' }}</button>
    </section>
    <template v-else>
      <section class="surface admin-panel">
        <h2>Precio general de inscripción</h2>
        <label>Inscripción (MXN) <input v-model.number="inscripcion" type="number" min="0" step="0.01" /></label>
        <button @click="guardarInscripcion">Guardar inscripción</button>
        <p class="muted">Poner $0 permite promociones sin costo. En registro solo aparece Sí/No.</p>
      </section>
      <section class="surface admin-panel">
        <h2>Membresías</h2>
        <div v-for="plan in planes" :key="plan.id" class="plan-row">
          <label>Nombre <input v-model="plan.nombre" maxlength="60" /></label>
          <label>Meses <input v-model.number="plan.duracion_meses" type="number" min="1" max="36" /></label>
          <label>Precio MXN <input v-model.number="plan.precio_pesos" type="number" min="0" step="0.01" /></label>
          <label>Activo <input v-model="plan.activo" type="checkbox" /></label>
          <button @click="guardarPlan(plan)">Guardar</button>
        </div>
        <h3>Nuevo plan</h3>
        <div class="plan-row">
          <label>Nombre <input v-model="nuevo.nombre" maxlength="60" /></label>
          <label>Meses <input v-model.number="nuevo.meses" type="number" min="1" max="36" /></label>
          <label>Precio MXN <input v-model.number="nuevo.precio_pesos" type="number" min="0" step="0.01" /></label>
          <button @click="crearPlan">Crear plan</button>
        </div>

      </section>
      <button class="lock-button" @click="autorizado=false;password=''">Bloquear administración</button>
    </template>
    <p role="status">{{ mensaje }}</p>
  </AppShell>
</template>
<style scoped>
.admin-panel{padding:25px;margin:18px 0;display:grid;gap:16px}
.plan-row{display:flex;align-items:end;flex-wrap:wrap;gap:12px;padding:16px 0;border-bottom:1px solid #343941}
label{display:grid;gap:8px;color:#b7bdc6;font-size:12px}
input{padding:12px;border:1px solid #343941;border-radius:9px;background:#111317;color:#fff}
input[type=checkbox]{width:24px;height:24px}
button{padding:12px 18px;border:0;border-radius:9px;background:#ee2046;color:#fff;cursor:pointer;font-weight:700}
.lock-button{background:#343941}
</style>
