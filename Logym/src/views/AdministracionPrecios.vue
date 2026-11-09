<script setup>
import { ref, computed, onMounted } from 'vue'
import AppShell from '../components/AppShell.vue'
import { adminConfigurado, crearPasswordAdmin, validarPasswordAdmin, listarPlanesAdmin, obtenerPrecioInscripcion, guardarPlanAdmin, guardarInscripcionAdmin, reporteFinanciero } from '../services/tauriApi'

const pestaña = ref('planes')
// Fechas locales para evitar desfases UTC en reportes de caja.
const fechaLocal = d => [d.getFullYear(), String(d.getMonth()+1).padStart(2,'0'), String(d.getDate()).padStart(2,'0')].join('-')
const hoy = () => new Date()
const rangoPredeterminado = periodo => {
  const fin = hoy()
  const inicio = new Date(fin.getFullYear(), fin.getMonth(), fin.getDate())
  if (periodo === 'semana') inicio.setDate(inicio.getDate()-6)
  if (periodo === 'mes') inicio.setDate(1)
  if (periodo === 'anual') inicio.setMonth(0,1)
  return [fechaLocal(inicio), fechaLocal(fin)]
}
const periodoFinanzas = ref('mes')
const [inicioInicial, finInicial] = rangoPredeterminado('mes')
const fechaInicio = ref(inicioInicial)
const fechaFin = ref(finInicial)
const resumenFinanzas = ref(null)
const errorFinanzas = ref('')
const cargandoFinanzas = ref(false)
const formatoFecha = valor => valor ? new Date(valor + 'T12:00:00').toLocaleDateString('es-MX',{day:'2-digit',month:'long',year:'numeric'}) : ''
const periodoMostrado = computed(() => resumenFinanzas.value ? `Del ${formatoFecha(resumenFinanzas.value.fecha_inicio)} al ${formatoFecha(resumenFinanzas.value.fecha_fin)} (ambos incluidos)` : '')
// Los reportes anuales se agrupan por meses calendario, nunca por cortes arbitrarios.
const barrasFinanzas = computed(() => {
  const reporte = resumenFinanzas.value
  if (!reporte) return []
  const inicio = new Date(reporte.fecha_inicio + 'T12:00:00')
  const fin = new Date(reporte.fecha_fin + 'T12:00:00')
  const dias = Math.round((fin - inicio) / 86400000) + 1
  if (dias < 1) return []
  const pagos = new Map(reporte.serie.map(p => [p.fecha, Number(p.total_centavos)]))
  if (periodoFinanzas.value === 'anual' || dias > 62) {
    const meses = []
    let cursor = new Date(inicio.getFullYear(), inicio.getMonth(), 1, 12)
    while (cursor <= fin) {
      const anio = cursor.getFullYear(), mes = cursor.getMonth()
      const total = reporte.serie.reduce((suma, p) => {
        const [y,m] = p.fecha.split('-').map(Number)
        return suma + (y === anio && m === mes+1 ? Number(p.total_centavos) : 0)
      }, 0)
      meses.push({ id: anio+'-'+mes, total, etiqueta: cursor.toLocaleDateString('es-MX',{month:'short',year:'numeric'}) })
      cursor = new Date(anio,mes+1,1,12)
    }
    return meses
  }
  return Array.from({length:dias},(_,i) => {
    const fecha = new Date(inicio)
    fecha.setDate(inicio.getDate()+i)
    const clave = fechaLocal(fecha)
    return { id:clave, total:pagos.get(clave)||0, etiqueta:fecha.toLocaleDateString('es-MX',{day:'2-digit',month:'short'}) }
  })
})
const unidadFinanzas = computed(() => periodoFinanzas.value === 'anual' || (resumenFinanzas.value && (new Date(resumenFinanzas.value.fecha_fin+'T12:00:00') - new Date(resumenFinanzas.value.fecha_inicio+'T12:00:00')) / 86400000 > 61) ? 'meses' : 'días')
const maxBarraFinanzas = computed(() => Math.max(1,...barrasFinanzas.value.map(p => p.total)))
const promedioFinanzas = computed(() => resumenFinanzas.value?.numero_pagos ? Math.round(resumenFinanzas.value.total_centavos / resumenFinanzas.value.numero_pagos) : 0)
const mejorBloqueFinanzas = computed(() => [...barrasFinanzas.value].sort((a,b)=>b.total-a.total)[0])
function seleccionarPeriodo(periodo) {
  periodoFinanzas.value = periodo
  if (periodo !== 'personalizado') [fechaInicio.value,fechaFin.value] = rangoPredeterminado(periodo)
  if (periodo !== 'personalizado') cargarFinanzas()
}
const configurado = ref(null)
const consultando = ref(false)
const autorizado = ref(false)
const password = ref('')
const confirmar = ref('')
const mensaje = ref('')
const planes = ref([])
const inscripcion = ref(0)
const nuevo = ref({ nombre:'', meses:1, precio_pesos:0, activo:true })
const moneda = n => (n/100).toLocaleString('es-MX',{style:'currency',currency:'MXN'})
const aCentavos = n => Math.round(Number(n)*100)
function mensajeErrorAdministracion(error, alternativa) {
  const texto = error instanceof Error ? error.message : error
  const permitidos = [
    'Las contraseñas no coinciden',
    'La contraseña debe tener entre 12 y 128 caracteres',
    'Ya existe una contraseña administrativa',
    'Configura primero la contraseña administrativa',
    'Contraseña incorrecta',
    'Datos de membresía inválidos',
    'Plan no encontrado',
    'Precio inválido',
    'Selecciona un rango de fechas válido (inicio no posterior al fin)',
    'Periodo inválido'
  ]
  return permitidos.includes(texto) ? texto : alternativa
}
async function cargar() {
  planes.value = (await listarPlanesAdmin(password.value)).map(p => ({...p, precio_pesos:p.precio_centavos/100}))
  inscripcion.value = (await obtenerPrecioInscripcion())/100
}
async function cargarFinanzas() {
  if (!autorizado.value) return
  if (!fechaInicio.value || !fechaFin.value || fechaInicio.value > fechaFin.value) {
    errorFinanzas.value = 'Selecciona una fecha inicial anterior o igual a la final.'
    return
  }
  errorFinanzas.value = ''
  cargandoFinanzas.value = true
  try { resumenFinanzas.value = await reporteFinanciero(fechaInicio.value, fechaFin.value, password.value) }
  catch (e) {
    console.error('Error al consultar el reporte financiero:', e)
    resumenFinanzas.value = null
    errorFinanzas.value = mensajeErrorAdministracion(e, 'No se pudo consultar el reporte financiero. Intenta nuevamente.')
  }
  finally { cargandoFinanzas.value = false }
}
function cambiarPestaña(valor) {
  pestaña.value = valor
  if (valor === 'finanzas') cargarFinanzas()
}
async function entrar() {
  if (configurado.value === null) return
  mensaje.value=''
  try {
    if (!configurado.value) {
      if (password.value !== confirmar.value) throw Error('Las contraseñas no coinciden')
      await crearPasswordAdmin(password.value)
      configurado.value=true
    } else await validarPasswordAdmin(password.value)
    autorizado.value=true
    await cargar()
  } catch(e) {
    console.error('Error al abrir la administración de precios:', e)
    mensaje.value = mensajeErrorAdministracion(e, 'No se pudo abrir la administración de precios. Intenta nuevamente.')
  }
}
async function guardarPlan(plan) {
  mensaje.value=''
  try {
    const { id, nombre, duracion_meses, meses, precio_pesos, activo } = plan
    await guardarPlanAdmin(password.value,{ id:id||null, nombre, meses:Number(duracion_meses||meses), precioCentavos:aCentavos(precio_pesos), activo })
    await cargar()
    mensaje.value='Plan guardado. Los cobros anteriores permanecen intactos.'
  } catch(e) {
    console.error('Error al guardar el plan:', e)
    mensaje.value = mensajeErrorAdministracion(e, 'No se pudo guardar el plan. Revisa los datos e intenta nuevamente.')
  }
}
async function crearPlan() {
  await guardarPlan(nuevo.value)
}
async function guardarInscripcion() {
  try {
    await guardarInscripcionAdmin(password.value,aCentavos(inscripcion.value))
    mensaje.value='Precio de inscripción actualizado para los próximos registros.'
  } catch(e) {
    console.error('Error al guardar el precio de inscripción:', e)
    mensaje.value = mensajeErrorAdministracion(e, 'No se pudo actualizar el precio de inscripción. Intenta nuevamente.')
  }
}
async function consultarEstadoAdmin() {
  consultando.value = true
  mensaje.value = ''
  configurado.value = null
  autorizado.value = false
  try {
    configurado.value = await adminConfigurado()
  } catch (e) {
    console.error('Error al consultar la configuración administrativa:', e)
    mensaje.value = 'No se pudo consultar la configuración administrativa. Intenta nuevamente.'
  } finally {
    consultando.value = false
  }
}
onMounted(consultarEstadoAdmin)
</script>

<template>
  <AppShell>
    <div class="mb-8 flex flex-wrap items-end justify-between gap-4">
      <div>
        <p class="eyebrow">CONFIGURACIÓN DEL NEGOCIO</p>
        <h1 class="page-title">Precios y membresías</h1>
        <p class="muted mt-3 text-sm">Los cambios afectan únicamente a nuevos cobros; el historial permanece intacto.</p>
      </div>
      <span class="rounded-full border border-[#343941] bg-[#171a20] px-4 py-2 text-xs font-semibold text-[#b7bdc6]">{{ autorizado ? 'Administración desbloqueada' : 'Acceso protegido' }}</span>
    </div>

    <section v-if="!autorizado && configurado === null" class="surface mx-auto grid max-w-lg gap-5 p-6 sm:p-8">
      <h2 class="text-xl font-bold">Configuración administrativa</h2>
      <p class="muted text-sm">{{ consultando ? 'Comprobando si ya existe una contraseña...' : 'No se pudo comprobar la contraseña administrativa. Revisa el error y vuelve a intentar.' }}</p>
      <button v-if="!consultando" class="rounded-xl bg-[#ee2046] px-5 py-3 font-bold text-white transition hover:bg-[#ff3157]" @click="consultarEstadoAdmin">Reintentar</button>
    </section>

    <section v-else-if="!autorizado" class="surface mx-auto grid max-w-lg gap-5 p-6 sm:p-8">
      <div>
        <p class="eyebrow">SEGURIDAD</p>
        <h2 class="mt-2 text-xl font-bold">{{ configurado ? 'Contraseña administrativa' : 'Crear contraseña administrativa' }}</h2>
        <p class="muted mt-2 text-sm">{{ configurado ? 'Introduce tu contraseña para administrar precios.' : 'Primera configuración: mínimo 12 caracteres. Guarda esta contraseña en un lugar seguro.' }}</p>
      </div>
      <form class="grid gap-4" @submit.prevent="entrar">
        <label class="grid gap-2 text-xs font-medium text-[#b7bdc6]">Contraseña
          <input v-model="password" type="password" autocomplete="off" required class="w-full rounded-xl border border-[#343941] bg-[#111317] px-4 py-3 text-sm text-white outline-none transition focus:border-[#ee2046] focus:ring-2 focus:ring-[#ee2046]/20" placeholder="Contraseña" />
        </label>
        <label v-if="!configurado" class="grid gap-2 text-xs font-medium text-[#b7bdc6]">Confirmar contraseña
          <input v-model="confirmar" type="password" autocomplete="off" required class="w-full rounded-xl border border-[#343941] bg-[#111317] px-4 py-3 text-sm text-white outline-none transition focus:border-[#ee2046] focus:ring-2 focus:ring-[#ee2046]/20" placeholder="Repite la contraseña" />
        </label>
        <button type="submit" class="rounded-xl bg-[#ee2046] px-5 py-3 font-bold text-white transition hover:bg-[#ff3157]">{{ configurado ? 'Entrar' : 'Crear contraseña' }}</button>
      </form>
    </section>

    <template v-else>

      <div class="mb-6 flex flex-wrap items-center justify-between gap-3 border-b border-white/10 pb-4">
        <div class="flex gap-2 rounded-xl bg-black/20 p-1" role="tablist" aria-label="Secciones administrativas">
          <button role="tab" :aria-selected="pestaña === 'planes'" @click="cambiarPestaña('planes')" class="rounded-lg px-5 py-2.5 text-sm font-semibold" :class="pestaña === 'planes' ? 'bg-[#ee2046] text-white' : 'text-zinc-400 hover:bg-white/10 hover:text-white'">Planes y precios</button>
          <button role="tab" :aria-selected="pestaña === 'finanzas'" @click="cambiarPestaña('finanzas')" class="rounded-lg px-5 py-2.5 text-sm font-semibold" :class="pestaña === 'finanzas' ? 'bg-[#ee2046] text-white' : 'text-zinc-400 hover:bg-white/10 hover:text-white'">Finanzas</button>
        </div>
        <button class="rounded-lg border border-white/10 px-4 py-2 text-xs font-semibold text-zinc-300 hover:bg-white/10" @click="autorizado=false;password='';resumenFinanzas=null;pestaña='planes'">Bloquear administración</button>
      </div>
      <section v-if="pestaña === 'finanzas'" class="space-y-5">
        <div class="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-[#ee2046]/20 bg-gradient-to-r from-[#30121c] via-[#19151b] to-[#111317] p-6">
          <div><p class="text-xs font-bold uppercase tracking-[.2em] text-[#ff6b85]">Control financiero</p><h2 class="mt-2 text-2xl font-black text-white">Ingresos del gimnasio</h2><p class="mt-2 text-sm text-zinc-400">Cobros reales, sin información de asistencias.</p></div>
          <select :value="periodoFinanzas" @change="seleccionarPeriodo($event.target.value)" aria-label="Periodo financiero" class="rounded-xl border border-white/10 bg-[#15171d] px-4 py-3 text-sm font-semibold text-white outline-none focus:border-[#ee2046]">
            <option value="dia">Hoy</option><option value="semana">Últimos 7 días</option><option value="mes">Mes actual</option><option value="anual">Año actual</option><option value="personalizado">Personalizado</option>
          </select>
        </div>
        <div v-if="periodoFinanzas === 'personalizado'" class="flex flex-wrap items-end gap-4 rounded-2xl border border-white/10 bg-[#15171d] p-4 sm:p-5">
          <label class="grid gap-2 text-xs font-semibold text-zinc-400">Desde
            <input v-model="fechaInicio" type="date" @change="periodoFinanzas='personalizado'" class="rounded-lg border border-white/15 bg-[#0d0f13] px-3 py-2.5 text-sm text-white outline-none focus:border-[#ee2046]" />
          </label>
          <label class="grid gap-2 text-xs font-semibold text-zinc-400">Hasta
            <input v-model="fechaFin" type="date" @change="periodoFinanzas='personalizado'" class="rounded-lg border border-white/15 bg-[#0d0f13] px-3 py-2.5 text-sm text-white outline-none focus:border-[#ee2046]" />
          </label>
          <button type="button" :disabled="cargandoFinanzas" @click="cargarFinanzas" class="rounded-lg bg-[#ee2046] px-5 py-2.5 text-sm font-bold text-white hover:bg-[#ff3157] disabled:opacity-50">{{ cargandoFinanzas ? 'Consultando…' : 'Consultar periodo' }}</button>
          <p v-if="resumenFinanzas" class="w-full text-sm font-medium text-zinc-300"><span class="text-[#ff6b85]">Periodo consultado:</span> {{ periodoMostrado }}</p>
        </div>
        <p v-if="resumenFinanzas && periodoFinanzas !== 'personalizado'" class="text-sm text-zinc-300"><span class="font-semibold text-[#ff6b85]">Periodo consultado:</span> {{ periodoMostrado }}</p>
        <p v-if="errorFinanzas" role="alert" class="text-sm text-red-300">{{ errorFinanzas }}</p>
        <div v-if="resumenFinanzas" class="grid gap-4 md:grid-cols-3">
          <article class="rounded-2xl border border-[#ee2046]/30 bg-gradient-to-br from-[#34101c] to-[#16151b] p-6">
            <p class="text-xs font-semibold uppercase tracking-wider text-zinc-300">Total cobrado</p>
            <strong class="mt-4 block text-3xl font-black text-white">{{ moneda(resumenFinanzas.total_centavos) }}</strong>
            <p class="mt-3 text-xs text-zinc-400">{{ resumenFinanzas.numero_pagos }} movimientos registrados</p>
          </article>
          <article class="rounded-2xl border border-white/10 bg-[#16191e] p-6">
            <p class="text-xs font-semibold uppercase tracking-wider text-zinc-400">Membresías</p>
            <strong class="mt-4 block text-3xl font-black text-white">{{ moneda(resumenFinanzas.membresias_centavos) }}</strong>
            <div class="mt-5 h-1.5 overflow-hidden rounded-full bg-white/10"><div class="h-full rounded-full bg-[#ee2046]" :style="{ width: resumenFinanzas.total_centavos ? (resumenFinanzas.membresias_centavos / resumenFinanzas.total_centavos * 100) + '%' : '0%' }"></div></div>
          </article>
          <article class="rounded-2xl border border-white/10 bg-[#16191e] p-6">
            <p class="text-xs font-semibold uppercase tracking-wider text-zinc-400">Inscripciones</p>
            <strong class="mt-4 block text-3xl font-black text-white">{{ moneda(resumenFinanzas.inscripciones_centavos) }}</strong>
            <div class="mt-5 h-1.5 overflow-hidden rounded-full bg-white/10"><div class="h-full rounded-full bg-[#ff9a54]" :style="{ width: resumenFinanzas.total_centavos ? (resumenFinanzas.inscripciones_centavos / resumenFinanzas.total_centavos * 100) + '%' : '0%' }"></div></div>
          </article>
        </div>
        <section v-if="resumenFinanzas" class="rounded-2xl border border-white/10 bg-[#15171d] p-5 sm:p-6">
          <div class="mb-5 flex flex-wrap items-center justify-between gap-3">
            <div><p class="text-xs font-bold uppercase tracking-widest text-[#ff6b85]">Análisis del periodo</p><h3 class="mt-1 text-lg font-bold text-white">¿Cuándo ingresó más dinero?</h3></div>
            <span class="text-xs text-zinc-400">Ingresos por {{ unidadFinanzas }} · {{ barrasFinanzas.length }} {{ unidadFinanzas }}</span>
          </div>
          <div class="mb-6 grid gap-3 sm:grid-cols-2">
            <div class="rounded-xl bg-black/25 p-4"><p class="text-xs text-zinc-400">Promedio por cobro</p><strong class="mt-1 block text-xl font-bold text-white">{{ moneda(promedioFinanzas) }}</strong></div>
            <div class="rounded-xl bg-black/25 p-4"><p class="text-xs text-zinc-400">{{ unidadFinanzas === 'meses' ? 'Mes con mayores ingresos' : 'Día con mayores ingresos' }}</p><strong class="mt-1 block text-lg font-bold text-white">{{ mejorBloqueFinanzas?.total ? mejorBloqueFinanzas.etiqueta : 'Sin cobros' }}</strong><p class="mt-1 text-xs text-[#ff6b85]">{{ moneda(mejorBloqueFinanzas?.total || 0) }}</p></div>
          </div>
          <div v-if="!resumenFinanzas.numero_pagos" class="rounded-xl border border-dashed border-white/10 p-8 text-center text-sm text-zinc-400">No hubo cobros durante este periodo. El total registrado es $0.</div>
          <div v-else class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <div v-for="barra in barrasFinanzas" :key="barra.id" class="rounded-xl border border-white/10 bg-[#101216] p-3">
              <div class="mb-2 flex items-center justify-between gap-2"><span class="text-[11px] text-zinc-400">{{ barra.etiqueta }}</span><strong class="text-xs text-white">{{ moneda(barra.total) }}</strong></div>
              <div class="h-3 overflow-hidden rounded-full bg-white/10"><div class="h-full rounded-full bg-gradient-to-r from-[#ee2046] to-[#ff728b] transition-all" :style="{width:(barra.total/maxBarraFinanzas*100)+'%'}"></div></div>
            </div>
          </div>
          <p class="mt-4 text-xs leading-relaxed text-zinc-500">El año se divide por meses calendario; los periodos cortos, por día. Los periodos sin cobros aparecen en cero.</p>
        </section>
      </section>
      <div v-if="pestaña === 'planes'" class="grid gap-5 xl:grid-cols-[minmax(280px,0.8fr)_minmax(0,1.5fr)]">
        <div class="grid content-start gap-5">
          <section class="surface grid gap-5 p-6">
            <div>
              <p class="eyebrow">CUOTA INICIAL</p>
              <h2 class="mt-2 text-xl font-bold">Inscripción</h2>
              <p class="muted mt-2 text-xs leading-relaxed">Este importe se agrega cuando seleccionas inscripción en el registro de un nuevo miembro.</p>
            </div>
            <label class="grid gap-2 text-xs font-medium text-[#b7bdc6]">Precio (MXN)
              <input v-model.number="inscripcion" type="number" min="0" step="0.01" class="w-full rounded-xl border border-[#343941] bg-[#111317] px-4 py-3 text-base text-white outline-none focus:border-[#ee2046] focus:ring-2 focus:ring-[#ee2046]/20" />
            </label>
            <button class="rounded-xl bg-[#ee2046] px-5 py-3 font-bold text-white transition hover:bg-[#ff3157]" @click="guardarInscripcion">Guardar inscripción</button>
          </section>
        </div>

        <section class="surface min-w-0 p-5 sm:p-6">
          <div class="mb-5">
            <p class="eyebrow">CATÁLOGO</p>
            <h2 class="mt-2 text-xl font-bold">Planes de membresía</h2>
            <p class="muted mt-2 text-xs">Edita la duración, el precio y la disponibilidad de cada plan.</p>
          </div>
          <div class="grid gap-3">
            <div v-for="plan in planes" :key="plan.id" class="grid gap-3 rounded-2xl border border-[#343941] bg-[#111317]/70 p-4 transition hover:border-[#505763]">
              <div class="flex items-center justify-between gap-3">
                <span class="text-xs font-semibold uppercase tracking-wider text-[#9298a2]">Plan #{{ plan.id }}</span>
                <span class="rounded-full px-3 py-1 text-[11px] font-semibold" :class="plan.activo ? 'bg-emerald-500/10 text-emerald-400' : 'bg-[#343941] text-[#9298a2]'">{{ plan.activo ? 'Activo' : 'Inactivo' }}</span>
              </div>
              <div class="grid gap-3 sm:grid-cols-[minmax(0,1.4fr)_90px_120px]">
                <label class="grid min-w-0 gap-2 text-xs text-[#b7bdc6]">Nombre
                  <input v-model="plan.nombre" maxlength="60" class="w-full min-w-0 rounded-lg border border-[#343941] bg-[#1b1e24] px-3 py-2.5 text-sm text-white outline-none focus:border-[#ee2046]" />
                </label>
                <label class="grid gap-2 text-xs text-[#b7bdc6]">Meses
                  <input v-model.number="plan.duracion_meses" type="number" min="1" max="36" class="w-full rounded-lg border border-[#343941] bg-[#1b1e24] px-3 py-2.5 text-sm text-white outline-none focus:border-[#ee2046]" />
                </label>
                <label class="grid gap-2 text-xs text-[#b7bdc6]">Precio MXN
                  <input v-model.number="plan.precio_pesos" type="number" min="0" step="0.01" class="w-full rounded-lg border border-[#343941] bg-[#1b1e24] px-3 py-2.5 text-sm text-white outline-none focus:border-[#ee2046]" />
                </label>
              </div>
              <div class="flex flex-wrap items-center justify-between gap-3">
                <label class="flex cursor-pointer items-center gap-2 text-xs text-[#b7bdc6]"><input v-model="plan.activo" type="checkbox" class="h-4 w-4 accent-[#ee2046]" /> Disponible para nuevos cobros</label>
                <button class="rounded-lg bg-[#ee2046] px-5 py-2.5 text-xs font-bold text-white transition hover:bg-[#ff3157]" @click="guardarPlan(plan)">Guardar cambios</button>
              </div>
            </div>
          </div>
          <div class="mt-6 rounded-2xl border border-dashed border-[#505763] bg-[#111317]/40 p-4">
            <h3 class="text-base font-bold">Crear nuevo plan</h3>
            <div class="mt-4 grid gap-3 sm:grid-cols-[minmax(0,1.4fr)_90px_120px]">
              <label class="grid min-w-0 gap-2 text-xs text-[#b7bdc6]">Nombre
                <input v-model="nuevo.nombre" maxlength="60" placeholder="Ej. Trimestral" class="w-full min-w-0 rounded-lg border border-[#343941] bg-[#1b1e24] px-3 py-2.5 text-sm text-white outline-none focus:border-[#ee2046]" />
              </label>
              <label class="grid gap-2 text-xs text-[#b7bdc6]">Meses
                <input v-model.number="nuevo.meses" type="number" min="1" max="36" class="w-full rounded-lg border border-[#343941] bg-[#1b1e24] px-3 py-2.5 text-sm text-white outline-none focus:border-[#ee2046]" />
              </label>
              <label class="grid gap-2 text-xs text-[#b7bdc6]">Precio MXN
                <input v-model.number="nuevo.precio_pesos" type="number" min="0" step="0.01" class="w-full rounded-lg border border-[#343941] bg-[#1b1e24] px-3 py-2.5 text-sm text-white outline-none focus:border-[#ee2046]" />
              </label>
            </div>
            <button class="mt-4 rounded-lg bg-[#ee2046] px-5 py-2.5 text-xs font-bold text-white transition hover:bg-[#ff3157]" @click="crearPlan">Crear plan</button>
          </div>
        </section>
      </div>
    </template>
    <p v-if="mensaje" role="status" aria-live="polite" class="mt-5 rounded-xl border border-[#343941] bg-[#171a20] px-4 py-3 text-sm text-[#d6dbe2]">{{ mensaje }}</p>
  </AppShell>
</template>
