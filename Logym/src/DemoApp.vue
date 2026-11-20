<script setup>
import { computed, reactive, ref } from 'vue'

const menu = [
  ['dashboard', 'Dashboard', '▦'], ['clientes', 'Clientes', '♙'],
  ['registro', 'Nuevo registro', '＋'], ['checkin', 'Check-in', '✓'],
  ['membresias', 'Membresías', '▤'], ['finanzas', 'Finanzas', '$'],
  ['administracion', 'Administración', '⚙']
]
const planes = [
  { id: 1, nombre: 'Mensual', precio: 350, dias: 30 },
  { id: 2, nombre: 'Estudiante', precio: 280, dias: 30 },
  { id: 3, nombre: 'Pareja', precio: 300, dias: 30 },
  { id: 4, nombre: 'Semanal', precio: 100, dias: 7 }
]
const hoy = () => new Date().toISOString().slice(0, 10)
const sumarDias = (fecha, dias) => {
  const d = new Date(fecha + 'T12:00:00'); d.setDate(d.getDate() + dias)
  return d.toISOString().slice(0, 10)
}
const inicial = () => [
  ['Alex Rivera', 1, -14], ['Mariana López', 2, -9], ['Carlos Mendoza', 1, -26],
  ['Sofía Hernández', 3, -5], ['Diego Torres', 4, -3], ['Valeria Cruz', 1, -33],
  ['Luis Ramírez', 1, -19], ['Fernanda Ruiz', 2, -38]
].map(([nombre, planId, inicio], i) => ({
  id: i + 1, nombre, planId, inicio: sumarDias(hoy(), inicio),
  fin: sumarDias(hoy(), inicio + planes.find(p => p.id === planId).dias),
  telefono: '', foto: '', activo: true
}))
const clientes = ref(inicial())
const movimientos = ref(clientes.value.map((c, i) => ({
  id: i + 1, fecha: c.inicio, cliente: c.nombre, concepto: 'Alta',
  monto: planes.find(p => p.id === c.planId).precio
})))
const asistencias = ref([{ clienteId: 1, fecha: hoy() }, { clienteId: 2, fecha: hoy() }, { clienteId: 4, fecha: hoy() }])
const pagina = ref('dashboard')
const busqueda = ref('')
const mensaje = ref('')
const modal = ref(false)
const nuevo = reactive({ nombre: '', telefono: '', planId: 1 })
const seleccionado = ref(null)
const planRenovacion = ref(1)
const periodo = ref('todos')
const titulo = computed(() => menu.find(m => m[0] === pagina.value)?.[1] || 'Dashboard')
const moneda = n => Number(n || 0).toLocaleString('es-MX', { style: 'currency', currency: 'MXN', maximumFractionDigits: 0 })
const planDe = id => planes.find(p => p.id === id) || planes[0]
const vigente = c => c.activo && c.inicio <= hoy() && c.fin >= hoy()
const activos = computed(() => clientes.value.filter(vigente).length)
const vencidos = computed(() => clientes.value.filter(c => !vigente(c)).length)
const totalIngresos = computed(() => movimientos.value.reduce((s, m) => s + m.monto, 0))
const lista = computed(() => clientes.value.filter(c => c.nombre.toLowerCase().includes(busqueda.value.toLowerCase())))
const movimientosVisibles = computed(() => movimientos.value.filter(m => periodo.value === 'todos' || m.fecha === hoy()))
function aviso(texto) { mensaje.value = texto; setTimeout(() => { if (mensaje.value === texto) mensaje.value = '' }, 4000) }
function navegar(p) { pagina.value = p; busqueda.value = ''; seleccionado.value = null; window.scrollTo(0, 0) }
function registrar() {
  if (!nuevo.nombre.trim()) return aviso('Escribe el nombre del cliente')
  const plan = planDe(Number(nuevo.planId))
  const c = { id: Math.max(0, ...clientes.value.map(c => c.id)) + 1, nombre: nuevo.nombre.trim(), telefono: nuevo.telefono, planId: plan.id, inicio: hoy(), fin: sumarDias(hoy(), plan.dias), foto: '', activo: true }
  clientes.value.unshift(c)
  movimientos.value.unshift({ id: Date.now(), fecha: hoy(), cliente: c.nombre, concepto: 'Alta', monto: plan.precio })
  nuevo.nombre = ''; nuevo.telefono = ''
  aviso('Cliente registrado correctamente (simulación)')
  navegar('clientes')
}
function renovar(c) {
  const plan = planDe(Number(planRenovacion.value))
  const inicio = c.fin >= hoy() ? sumarDias(c.fin, 1) : hoy()
  c.planId = plan.id; c.inicio = inicio; c.fin = sumarDias(inicio, plan.dias - 1); c.activo = true
  movimientos.value.unshift({ id: Date.now(), fecha: hoy(), cliente: c.nombre, concepto: 'Renovación', monto: plan.precio })
  seleccionado.value = null
  aviso('Renovación simulada: ' + c.nombre)
}
function checkin(c) {
  if (!vigente(c)) return aviso('Acceso denegado: membresía no vigente')
  if (asistencias.value.some(a => a.clienteId === c.id && a.fecha === hoy())) return aviso('Asistencia ya registrada hoy')
  asistencias.value.unshift({ clienteId: c.id, fecha: hoy() })
  aviso('Acceso autorizado: ' + c.nombre)
}
function reiniciar() {
  if (!window.confirm('¿Reiniciar todos los datos ficticios de esta demo?')) return
  clientes.value = inicial()
  movimientos.value = clientes.value.map((c, i) => ({ id: i + 1, fecha: c.inicio, cliente: c.nombre, concepto: 'Alta', monto: planDe(c.planId).precio }))
  asistencias.value = [{ clienteId: 1, fecha: hoy() }, { clienteId: 2, fecha: hoy() }, { clienteId: 4, fecha: hoy() }]
  navegar('dashboard')
  aviso('Datos de demostración restablecidos')
}
</script>

<template>
  <div class="demo-app">
    <aside class="demo-sidebar">
      <div class="demo-brand"><div class="demo-mark">V</div><div><strong>VIRTUS GYM</strong><small>SISTEMA DE GESTIÓN</small></div></div>
      <div class="demo-nav-label">MENÚ PRINCIPAL</div>
      <nav aria-label="Navegación principal">
        <button v-for="m in menu" :key="m[0]" :class="{ selected: pagina === m[0] }" @click="navegar(m[0])"><span class="demo-icon">{{ m[2] }}</span>{{ m[1] }}</button>
      </nav>
      <div class="demo-side-bottom"><span class="demo-dot"></span> Modo demostración <small>Sin conexión a datos reales</small></div>
    </aside>
    <main class="demo-main">
      <header class="demo-top"><div><span class="demo-kicker">VIRTUS GYM / {{ titulo.toUpperCase() }}</span><h1>{{ titulo }}</h1></div><span class="demo-pill">● DEMO WEB</span></header>
      <div class="demo-mobile-nav"><button v-for="m in menu" :key="m[0]" :class="{ selected: pagina === m[0] }" @click="navegar(m[0])">{{ m[1] }}</button></div>
      <div class="demo-notice">Esta es una demostración interactiva con información ficticia. No utiliza licencia, lector biométrico ni base de datos.</div>

      <template v-if="pagina === 'dashboard'">
        <div class="demo-welcome"><div><span class="demo-kicker">RESUMEN GENERAL</span><h2>Control total de tu gimnasio.</h2><p>Monitorea membresías, ingresos y accesos desde un solo lugar.</p></div><button class="demo-primary" @click="navegar('registro')">+ Registrar cliente</button></div>
        <div class="demo-stats">
          <div class="demo-stat"><span>Clientes registrados</span><strong>{{ clientes.length }}</strong><small>Expedientes en el sistema</small></div>
          <div class="demo-stat"><span>Miembros activos</span><strong>{{ activos }}</strong><small>Membresías vigentes</small></div>
          <div class="demo-stat"><span>Entradas de hoy</span><strong>{{ asistencias.filter(a => a.fecha === hoy()).length }}</strong><small>Check-ins simulados</small></div>
          <div class="demo-stat"><span>Ingresos registrados</span><strong>{{ moneda(totalIngresos) }}</strong><small>Movimientos de la demo</small></div>
        </div>
        <div class="demo-columns"><section class="demo-panel"><div class="demo-panel-head"><h3>Miembros recientes</h3><button class="demo-link" @click="navegar('clientes')">Ver todos →</button></div><div v-for="c in clientes.slice(0,5)" :key="c.id" class="demo-list-row"><div class="demo-avatar">{{ c.nombre[0] }}</div><div class="demo-grow"><b>{{ c.nombre }}</b><small>{{ planDe(c.planId).nombre }} · Vence {{ c.fin }}</small></div><span :class="['demo-status', vigente(c) ? 'ok' : 'bad']">{{ vigente(c) ? 'Vigente' : 'Vencida' }}</span></div></section>
          <section class="demo-panel"><div class="demo-panel-head"><h3>Estado de membresías</h3></div><div class="demo-big-number">{{ activos }} <span>/ {{ clientes.length }}</span></div><div class="demo-progress"><div :style="{width: clientes.length ? activos/clientes.length*100+'%' : '0%'}"></div></div><p class="demo-muted">{{ vencidos }} membresías vencidas o inactivas</p><button class="demo-outline" @click="navegar('checkin')">Ir al Check-in</button></section></div>
      </template>

      <template v-else-if="pagina === 'clientes'">
        <section class="demo-panel"><div class="demo-panel-head"><h3>Directorio de clientes <span class="demo-count">{{ clientes.length }}</span></h3><button class="demo-primary" @click="navegar('registro')">+ Nuevo cliente</button></div><input v-model="busqueda" class="demo-input" placeholder="Buscar por nombre..." aria-label="Buscar clientes"/><div v-for="c in lista" :key="c.id" class="demo-client-row"><div class="demo-avatar">{{ c.nombre[0] }}</div><div class="demo-grow"><b>{{ c.nombre }}</b><small>#{{ c.id }} · {{ planDe(c.planId).nombre }} · Vence {{ c.fin }}</small></div><span :class="['demo-status', vigente(c) ? 'ok' : 'bad']">{{ vigente(c) ? 'Vigente' : 'Vencida' }}</span><button class="demo-outline" @click="seleccionado=c;planRenovacion=c.planId">Renovar</button></div><p v-if="!lista.length" class="demo-muted">No hay clientes con ese nombre.</p></section>
      </template>

      <template v-else-if="pagina === 'registro'">
        <section class="demo-panel demo-form"><div class="demo-panel-head"><h3>Información del miembro</h3></div><label>Nombre completo<input v-model="nuevo.nombre" class="demo-input" placeholder="Ej. Alex Rivera" maxlength="100"/></label><label>Teléfono (opcional)<input v-model="nuevo.telefono" class="demo-input" placeholder="10 dígitos" maxlength="10"/></label><label>Tipo de membresía<select v-model.number="nuevo.planId" class="demo-input"><option v-for="p in planes" :key="p.id" :value="p.id">{{ p.nombre }} — {{ moneda(p.precio) }}</option></select></label><div class="demo-fake-hardware">◎ Captura biométrica desactivada en la demostración</div><div class="demo-total"><span>Total de membresía</span><strong>{{ moneda(planDe(nuevo.planId).precio) }}</strong></div><button class="demo-primary" @click="registrar">Guardar registro simulado</button></section>
      </template>

      <template v-else-if="pagina === 'checkin'">
        <section class="demo-panel"><div class="demo-panel-head"><h3>Control de acceso</h3><span class="demo-pill">SIMULACIÓN</span></div><div class="demo-fake-hardware demo-check-icon">◎</div><p class="demo-muted">Selecciona un cliente y simula su entrada. No se requiere lector de huellas.</p><input v-model="busqueda" class="demo-input" placeholder="Buscar cliente..." aria-label="Buscar cliente para check-in"/><div v-for="c in lista" :key="c.id" class="demo-client-row"><div class="demo-avatar">{{ c.nombre[0] }}</div><div class="demo-grow"><b>{{ c.nombre }}</b><small>{{ planDe(c.planId).nombre }} · {{ c.fin }}</small></div><button class="demo-primary" @click="checkin(c)">Simular entrada</button></div></section>
      </template>

      <template v-else-if="pagina === 'membresias'">
        <div class="demo-plan-grid"><section v-for="p in planes" :key="p.id" class="demo-panel demo-plan"><span class="demo-kicker">PLAN DE MEMBRESÍA</span><h3>{{ p.nombre }}</h3><strong>{{ moneda(p.precio) }}</strong><p class="demo-muted">Duración: {{ p.dias }} días</p><button class="demo-outline" @click="nuevo.planId=p.id;navegar('registro')">Registrar con este plan</button></section></div>
      </template>

      <template v-else-if="pagina === 'finanzas'">
        <div class="demo-stats"><div class="demo-stat"><span>Ingresos totales</span><strong>{{ moneda(totalIngresos) }}</strong></div><div class="demo-stat"><span>Movimientos</span><strong>{{ movimientos.length }}</strong></div><div class="demo-stat"><span>Renovaciones</span><strong>{{ movimientos.filter(m=>m.concepto==='Renovación').length }}</strong></div><div class="demo-stat"><span>Altas</span><strong>{{ movimientos.filter(m=>m.concepto==='Alta').length }}</strong></div></div><section class="demo-panel"><div class="demo-panel-head"><h3>Historial de movimientos</h3><select v-model="periodo" class="demo-input demo-filter"><option value="todos">Todos</option><option value="hoy">Hoy</option></select></div><div v-for="m in movimientosVisibles" :key="m.id" class="demo-client-row"><div class="demo-grow"><b>{{ m.cliente }}</b><small>{{ m.fecha }} · {{ m.concepto }}</small></div><strong>{{ moneda(m.monto) }}</strong></div><p v-if="!movimientosVisibles.length" class="demo-muted">Sin movimientos en este periodo.</p></section>
      </template>

      <template v-else-if="pagina === 'administracion'">
        <section class="demo-panel demo-form"><h3>Configuración de demostración</h3><p class="demo-muted">Esta versión no utiliza contraseñas, licencias, hardware ni información real. Los cambios son temporales y se pierden al recargar la página.</p><div class="demo-list-row"><div class="demo-grow"><b>Estado del sistema</b><small>Demo web — Vue 3 + Vite</small></div><span class="demo-status ok">Activo</span></div><button class="demo-outline" @click="reiniciar">Restablecer datos ficticios</button></section>
      </template>
      <footer class="demo-footer">VIRTUS GYM · Demostración comercial · Datos ficticios</footer>
    </main>
    <div v-if="seleccionado" class="demo-overlay" @click.self="seleccionado=null"><section class="demo-modal"><div class="demo-panel-head"><h3>Renovar membresía</h3><button class="demo-close" @click="seleccionado=null">✕</button></div><p class="demo-muted">{{ seleccionado.nombre }}</p><label>Selecciona un plan<select v-model.number="planRenovacion" class="demo-input"><option v-for="p in planes" :key="p.id" :value="p.id">{{ p.nombre }} — {{ moneda(p.precio) }}</option></select></label><div class="demo-actions"><button class="demo-outline" @click="seleccionado=null">Cancelar</button><button class="demo-primary" @click="renovar(seleccionado)">Confirmar renovación simulada</button></div></section></div>
    <div v-if="mensaje" class="demo-toast" role="status">{{ mensaje }}</div>
  </div>
</template>

<style>
:root{font-family:Inter,ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif;color:#f4f4f5;background:#090a0c;font-synthesis:none}*{box-sizing:border-box}body{margin:0}button,input,select{font:inherit}button{cursor:pointer}.demo-app{display:flex;min-height:100vh;background:#090a0c;color:#eee}.demo-sidebar{width:255px;flex-shrink:0;background:#101114;border-right:1px solid #28292d;padding:27px 14px;display:flex;flex-direction:column;position:sticky;top:0;height:100vh}.demo-brand{display:flex;align-items:center;gap:12px;padding:6px 11px 35px}.demo-brand strong{font-size:16px;letter-spacing:2px}.demo-brand small{display:block;font-size:9px;letter-spacing:2px;color:#8d8f97;margin-top:5px}.demo-mark{height:40px;width:40px;border:1px solid #c8a76c;border-radius:11px;display:grid;place-items:center;font-family:Georgia,serif;font-size:25px;color:#d6b87c}.demo-nav-label,.demo-kicker{font-size:10px;letter-spacing:1.7px;color:#a6a7ad;font-weight:700}.demo-nav-label{padding:0 14px 14px;color:#777981}.demo-sidebar nav{display:grid;gap:5px}.demo-sidebar nav button{border:0;background:transparent;color:#a3a4ad;text-align:left;padding:13px 14px;border-radius:10px;display:flex;gap:12px;align-items:center;font-size:13px}.demo-sidebar nav button:hover,.demo-sidebar nav button.selected{background:#262626;color:#f5e0b5}.demo-sidebar nav button.selected{border-left:2px solid #c5a46c}.demo-icon{width:19px;text-align:center;font-size:19px}.demo-side-bottom{margin-top:auto;border-top:1px solid #303036;padding:20px 12px;color:#d5bb83;font-size:12px}.demo-side-bottom small{display:block;margin-top:7px;color:#777981}.demo-dot{display:inline-block;width:7px;height:7px;border-radius:50%;background:#d2b57c;margin-right:6px}.demo-main{flex:1;min-width:0;max-width:1550px;padding:34px clamp(18px,4vw,65px)}.demo-top{display:flex;justify-content:space-between;align-items:center;margin-bottom:25px}.demo-top h1{font-size:30px;margin:6px 0 0;letter-spacing:-.8px}.demo-pill{color:#ddc08d;border:1px solid #6a5434;background:#2b251c;padding:7px 11px;border-radius:30px;font-size:10px;letter-spacing:1.2px;font-weight:700;white-space:nowrap}.demo-notice{border:1px solid #49402c;background:#211d15;border-radius:9px;padding:12px 15px;color:#d7c19b;font-size:12px;margin-bottom:25px}.demo-welcome{display:flex;align-items:center;justify-content:space-between;gap:18px;padding:27px 0}.demo-welcome h2{font-size:28px;margin:10px 0}.demo-welcome p,.demo-muted{color:#a1a1aa;font-size:13px;line-height:1.7}.demo-primary,.demo-outline{border-radius:9px;padding:11px 16px;font-weight:650;font-size:12px;white-space:nowrap}.demo-primary{background:#c9a970;border:1px solid #c9a970;color:#19160f}.demo-primary:hover{background:#e2c38b}.demo-outline{background:#191a1d;border:1px solid #45464b;color:#eee}.demo-outline:hover{border-color:#bfa16f}.demo-link{background:none;border:0;color:#d9b77e;font-size:12px}.demo-stats{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:14px;margin-bottom:23px}.demo-stat,.demo-panel{background:#151619;border:1px solid #2b2c31;border-radius:13px}.demo-stat{padding:23px 20px}.demo-stat span{display:block;color:#aaaab2;font-size:12px}.demo-stat strong{display:block;font-size:29px;letter-spacing:-1px;margin:13px 0}.demo-stat small{font-size:11px;color:#787a84}.demo-columns{display:grid;grid-template-columns:1.5fr 1fr;gap:18px}.demo-panel{padding:24px;margin-bottom:20px}.demo-panel-head{display:flex;justify-content:space-between;align-items:center;gap:12px;margin-bottom:22px}.demo-panel h3{font-size:16px;margin:0}.demo-list-row,.demo-client-row{display:flex;align-items:center;gap:14px;padding:14px 0;border-top:1px solid #2b2c31}.demo-grow{flex:1;min-width:0}.demo-grow b,.demo-grow small{display:block}.demo-grow b{font-size:13px}.demo-grow small{font-size:11px;color:#92939b;margin-top:5px}.demo-avatar{width:39px;height:39px;display:grid;place-items:center;flex-shrink:0;border-radius:10px;background:#383024;color:#e8c995;font-weight:750}.demo-status{font-size:11px;padding:5px 9px;border-radius:6px;white-space:nowrap}.demo-status.ok{background:#193329;color:#83d3a8}.demo-status.bad{background:#392322;color:#e9a3a0}.demo-big-number{font-size:52px;font-weight:800;margin:18px 0}.demo-big-number span{font-size:20px;color:#85858d}.demo-progress{height:8px;background:#353535;border-radius:9px;overflow:hidden}.demo-progress div{height:100%;background:#c9a970}.demo-count{font-size:11px;color:#d8bd89}.demo-input{background:#0e0f11;border:1px solid #383a40;border-radius:9px;padding:12px;color:#f4f4f5;width:100%;margin:8px 0 20px;outline:none}.demo-input:focus{border-color:#c9a970}.demo-form{max-width:650px}.demo-form label,.demo-modal label{display:block;font-size:12px;color:#c6c6cb;margin-top:14px}.demo-fake-hardware{padding:25px;border:1px dashed #5b513c;border-radius:10px;text-align:center;color:#c9b58f;font-size:13px;margin:10px 0 20px}.demo-check-icon{font-size:75px;padding:35px}.demo-total{display:flex;justify-content:space-between;align-items:center;padding:18px 0;border-top:1px solid #333;margin-bottom:15px}.demo-total strong{font-size:23px;color:#e1bf85}.demo-plan-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:18px}.demo-plan h3{margin:15px 0}.demo-plan>strong{display:block;font-size:30px;color:#e0be86}.demo-filter{max-width:135px;margin:0}.demo-footer{margin-top:35px;padding:20px 0;border-top:1px solid #292a2e;color:#777881;font-size:11px}.demo-overlay{position:fixed;inset:0;background:#000b;display:grid;place-items:center;padding:20px;z-index:200}.demo-modal{background:#191a1d;border:1px solid #45464b;border-radius:14px;padding:25px;width:min(100%,440px)}.demo-close{border:0;background:none;color:#ddd;font-size:20px}.demo-actions{display:flex;gap:10px;justify-content:flex-end;margin-top:20px}.demo-toast{position:fixed;right:20px;top:20px;z-index:300;background:#233a2d;border:1px solid #6ca17d;color:#d9ffe6;padding:15px 20px;border-radius:10px;box-shadow:0 12px 35px #0008;font-size:13px}.demo-mobile-nav{display:none}@media(max-width:1100px){.demo-stats{grid-template-columns:repeat(2,minmax(0,1fr))}.demo-sidebar{width:210px}.demo-main{padding:25px}}@media(max-width:760px){.demo-app{display:block}.demo-sidebar{display:none}.demo-main{padding:18px 14px}.demo-mobile-nav{display:flex;overflow:auto;gap:7px;margin-bottom:16px;padding-bottom:7px}.demo-mobile-nav button{border:1px solid #39393b;background:#18191c;color:#c9c9d0;border-radius:8px;padding:9px 11px;white-space:nowrap;font-size:12px}.demo-mobile-nav button.selected{background:#4b3b23;color:#f0d6a6}.demo-top h1{font-size:24px}.demo-top{margin-bottom:17px}.demo-welcome{display:block;padding:10px 0 20px}.demo-welcome h2{font-size:23px}.demo-welcome .demo-primary{margin-top:8px}.demo-columns,.demo-plan-grid{grid-template-columns:1fr}.demo-stat{padding:17px}.demo-stat strong{font-size:23px}.demo-panel{padding:17px}.demo-client-row{flex-wrap:wrap}.demo-client-row .demo-outline{margin-left:auto}.demo-actions{flex-wrap:wrap}.demo-actions button{flex:1}.demo-notice{line-height:1.6}}
</style>