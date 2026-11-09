import test from 'node:test'
import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { SourceTextModule, SyntheticModule, createContext } from 'node:vm'
import * as Vue from 'vue'
import { MENSAJES_LECTOR, mensajeErrorLector, detalleLectorPublico } from '../src/services/mensajesLector.js'
import { duracionPlan, fechaFinPlan, ordenarPlanes } from '../src/services/duracionPlanes.js'
import { ventanaSieteDias, filtrarCriticas } from '../src/services/periodosDashboard.js'

test('Catálogo: Mensual primero sin cambiar el orden de los demás ni mutar los datos', () => {
  const planes=[{id:7,nombre:'Estudiante'},{id:2,nombre:' MENSUAL '},{id:8,nombre:'Semanal'}]
  assert.deepEqual(ordenarPlanes(planes).map(p=>p.id),[2,7,8])
  assert.deepEqual(planes.map(p=>p.id),[7,2,8])
  assert.deepEqual(ordenarPlanes([planes[0],planes[2]]).map(p=>p.id),[7,8])
})

test('Registro y renovación abren con Mensual; un reintento conserva el plan confirmado', async () => {
  const catalogo=[{id:7,nombre:'Estudiante',duracion_meses:1},{id:2,nombre:'Mensual',duracion_meses:1}]
  const api={listarTiposMembresia:async()=>catalogo}
  const f=await registro({api});assert.equal(f.state.tipoMembresiaId.value,'2');assert.equal(f.state.tiposMembresia.value[0].id,2);await f.unmount()
  const g=await fixture('composables/useRenovacion.js',{api});const r=g.ns.useRenovacion();await r.abrir({id:1})
  assert.equal(r.planId.value,'2');assert.equal(r.planes.value[0].id,2)
  const pendiente=JSON.stringify({id:'a'.repeat(32),clienteId:1,planId:7})
  const h=await fixture('composables/useRenovacion.js',{api,storage:{getItem:()=>pendiente}});const s=h.ns.useRenovacion();await s.abrir({id:1})
  assert.equal(s.planes.value[0].id,2);assert.equal(s.planId.value,'7')
})

test('Dashboard: siete días cronológicos con ceros, fin de mes, año y fecha local', () => {
  for (const hoy of [new Date(2026,10,14,23),new Date(2026,10,11,0),new Date(2027,0,2,12),new Date(2024,2,2,12)]) {
    const dias=ventanaSieteDias([],hoy);assert.equal(dias.length,7)
    const conDatos=ventanaSieteDias([{fecha:dias[0].fecha,total:2},{fecha:dias[6].fecha,total:1},{fecha:'2000-01-01',total:100}],hoy)
    assert.deepEqual(conDatos.map(d=>d.total),[2,0,0,0,0,0,1]);assert.equal(new Set(dias.map(d=>d.fecha)).size,7)
    for(let i=1;i<7;i++)assert.ok(dias[i].fecha>dias[i-1].fecha)
    assert.equal(new Date(dias[6].fecha+'T12:00:00').getDate(),hoy.getDate())
  }
})

test('Dashboard: filtro de vencidas calendario no elimina próximas ni inactivos de sus registros', () => {
  const clientes=[['2026-10-30','vencida'],['2026-09-30','vencida'],['2026-08-30','vencida'],['2026-07-30','vencida'],['2026-12-01','por_vencer'],['2026-10-30','activa']].map(([fecha_fin_membresia,estado_membresia],id)=>({id,fecha_fin_membresia,estado_membresia,activo:true}))
  clientes.push({...clientes[0],id:9,activo:false})
  for(const [meses,ids] of [['1',[0,4]],['2',[0,1,4]],['3',[0,1,2,4]],['todas',[0,1,2,3,4]]])assert.deepEqual(filtrarCriticas(clientes,meses,new Date(2026,10,30)).map(c=>c.id),ids)
  assert.equal(clientes.length,7)
})

test('Renovación común: respuesta perdida y reinicio reenvían exactamente una operación persistida', async () => {
  const data=new Map(),envios=[],storage={getItem:k=>data.get(k)||null,setItem:(k,v)=>data.set(k,v),removeItem:k=>data.delete(k)}
  const f=await fixture('composables/useRenovacion.js',{storage,api:{renovarMembresia:async(...args)=>{envios.push(args);throw Error('SQLCipher C:\\secreto\\gym.db')}}})
  const r=f.ns.useRenovacion();await r.abrir({id:1});await r.confirmar();assert.equal(r.pendiente.value.clienteId,1)
  assert.ok(!r.error.value.includes('SQLCipher'));assert.equal(f.erroresTecnicos.length,1)
  r.cerrar();const g=await fixture('composables/useRenovacion.js',{storage,api:{renovarMembresia:async(...args)=>{envios.push(args);return{id:1}}}})
  const s=g.ns.useRenovacion();await s.abrir({id:999});assert.equal(s.cliente.value.id,1);await s.confirmar()
  assert.deepEqual(envios[0],envios[1]);assert.equal(data.size,0);assert.equal(s.abierta.value,false)
})

test('Renovación común: almacenamiento no disponible impide enviar un cobro sin protección', async () => {
  const f=await fixture('composables/useRenovacion.js',{storage:{getItem:()=>null,setItem:()=>{throw Error('sin almacenamiento')}}})
  const s=f.ns.useRenovacion();await s.abrir({id:1});await s.confirmar();assert.equal(s.pendiente.value,null);assert.equal(s.guardando.value,false)
  assert.ok(!f.calls.some(([c])=>c==='toast'));assert.ok(s.error.value)
})

test('Renovación común: rechazo sin escritura permite cambiar plan; cerrar impide nuevos cobros', async () => {
  const envios=[]
  const f=await fixture('composables/useRenovacion.js',{api:{renovarMembresia:async(...args)=>{envios.push(args);if(envios.length===1)throw 'Selecciona una membresía vigente';return{id:1}}}})
  const s=f.ns.useRenovacion();await s.abrir({id:1});await s.confirmar();assert.equal(s.pendiente.value,null);assert.equal(s.guardando.value,false)
  await s.confirmar();assert.equal(envios.length,2);assert.notEqual(envios[0][2],envios[1][2]);await s.confirmar();assert.equal(envios.length,2)
})

test('Dashboard: evento confirmado actualiza clientes, lista crítica y métricas sin polling', async () => {
  let vigente=false
  const f=await fixture('composables/useDashboard.js',{api:{listarClientes:async()=>[],listarClientesMembresia:async()=>[{id:1,activo:true,estado_membresia:vigente?'activa':'por_vencer'}],obtenerMetricasDashboard:async()=>({total_clientes:1})}})
  const s=f.ns.useDashboard();await f.mount();assert.equal(s.alertasMembresia.value.length,1);assert.equal(s.antiguedadVencidas.value,'3')
  vigente=true;await f.listeners.get('logym://datos-actualizados')();assert.equal(s.alertasMembresia.value.length,0);await f.unmount();assert.equal(f.listeners.size,0)
})

test('Registro: fecha manipulada en Vue no se envía como fecha de alta', async () => {
  const f=await registro();const s=f.state;s.sinHuella.value=true;s.nombre.value='Sintético';s.fechaRegistro.value='2099-01-01';await s.guardar()
  const datos=f.calls.find(([c])=>c==='guardar')[1];assert.equal('fechaRegistro' in datos,false);await f.unmount()
})

test('Duraciones: siete días inclusivos, bisiestos y meses históricos', () => {
  for (const [inicio, plan, fin] of [
    ['2026-11-08', { duracion_dias:7 }, '2026-11-14'],
    ['2024-02-26', { duracion_dias:7 }, '2024-03-03'],
    ['2026-12-29', { duracion_dias:7 }, '2027-01-04'],
    ['2026-11-08', { duracion_dias:1 }, '2026-11-08'],
    ['2024-01-31', { duracion_meses:1 }, '2024-02-29']
  ]) assert.equal(fechaFinPlan(inicio,plan),fin)
  assert.equal(duracionPlan({duracion_dias:7}),'7 días')
  assert.equal(duracionPlan({duracion_meses:1}),'1 mes')
  assert.equal(fechaFinPlan('2026-11-08',{duracion_dias:0}),'')
})

test('Visita: nombre opcional, una petición por doble clic y nueva entrada con otro ID', async () => {
  let completar
  const envios = []
  const f = await fixture('composables/useVisita.js', {api:{registrarVisita:async(id,nombre)=>{
    envios.push({id,nombre}); await new Promise(resolve=>{completar=resolve}); return {id,total_centavos:5000}
  }}})
  const s=f.ns.useVisita(); s.abrir()
  const primera=s.guardar(); await s.guardar()
  assert.equal(envios.length,1); assert.equal(s.guardando.value,true)
  assert.equal(envios[0].nombre,''); assert.match(envios[0].id,/^[a-f0-9]{32}$/)
  completar(); assert.equal((await primera).total_centavos,5000)
  s.abrir(); s.nombre.value=' Visitante sintético '
  const otra=s.guardar(); completar(); await otra
  assert.notEqual(envios[0].id,envios[1].id); assert.equal(envios[1].nombre,'Visitante sintético')
})

test('Visita: respuesta fallida conserva ID y nombre para reintentar sin exponer diagnósticos', async () => {
  const envios=[]
  const f=await fixture('composables/useVisita.js',{api:{registrarVisita:async(id,nombre)=>{
    envios.push({id,nombre}); if(envios.length===1)throw Error('SQLCipher C:\\privado\\gym.db: 59'); return {id,total_centavos:5000}
  }}})
  const s=f.ns.useVisita();s.nombre.value='Sintético'; assert.equal(await s.guardar(),null)
  assert.equal(s.guardando.value,false);assert.equal(s.porConfirmar.value,true)
  assert.ok(!/SQLCipher|privado|59|gym\.db/.test(s.mensaje.value));assert.equal(f.erroresTecnicos.length,1)
  s.abrir(); assert.equal(s.nombre.value,'Sintético');await s.guardar()
  assert.deepEqual(envios[0],envios[1]);assert.equal(s.porConfirmar.value,false)
})

test('Visita: un reinicio conserva solo el ID pendiente y recupera la operación', async () => {
  const data=new Map();const storage={getItem:k=>data.get(k)||null,setItem:(k,v)=>data.set(k,v),removeItem:k=>data.delete(k)}
  const f=await fixture('composables/useVisita.js',{storage,api:{registrarVisita:async()=>{throw Error('respuesta perdida')}}})
  const s=f.ns.useVisita();s.nombre.value='Nombre privado';await s.guardar()
  assert.equal(data.size,1);const id=[...data.values()][0];assert.match(id,/^[a-f0-9]{32}$/);assert.ok(!JSON.stringify([...data]).includes('Nombre privado'))
  const g=await fixture('composables/useVisita.js',{storage});const t=g.ns.useVisita();t.abrir()
  assert.equal(t.porConfirmar.value,true);await t.guardar()
  assert.deepEqual(g.calls[0],['visita',id,'']);assert.equal(data.size,0)
})

test('Registro semanal con y sin huella conserva inscripción opcional y fecha inclusiva', async () => {
  for(const sinHuella of [false,true]){
    const f=await registro({api:{listarTiposMembresia:async()=>[{id:7,nombre:'Semanal',duracion_meses:1,duracion_dias:7,precio_centavos:12000}]}})
    const s=f.state;s.sinHuella.value=sinHuella;s.nombre.value='Semanal sintético';s.fechaRegistro.value='2026-11-08';s.fechaInicioMembresia.value='2026-11-08';s.tipoMembresiaId.value='7';s.incluirInscripcion.value=false
    await Vue.nextTick();assert.equal(s.fechaFinMembresia.value,'2026-11-14')
    if(!sinHuella){await s.capturarHuellaCliente(1);await s.capturarHuellaCliente(2)}
    await s.guardar();const envio=f.calls.find(([c])=>c==='guardar')[1]
    assert.equal(envio.fechaFinMembresia,'2026-11-14');assert.equal(envio.incluirInscripcion,false);assert.equal(envio.sinHuella,sinHuella)
    await f.unmount()
  }
})

// Ejecuta los módulos de producción. Se simulan únicamente sus fronteras:
// IPC, cámara, router, lifecycle y reloj; no se replica la lógica de negocio.
async function fixture(file, overrides = {}) {
  const mounted = [], unmounted = [], listeners = new Map(), calls = [], erroresTecnicos = []
  let now = 0, timerId = 0
  const timers = new Map()
  const api = {
    pausarLectorGlobal: async v => { calls.push(['pausa', v]) },
    inicializarBaseDeDatos: async () => {},
    listarTiposMembresia: async () => [{ id: 1, nombre: 'Mensual', duracion_meses: 1, solo_alta: false }],
    listarClientesMembresia: async () => [{ id:1,nombre:'Sintético',activo:true }],
    obtenerPrecioInscripcion: async () => 15000,
    buscarClientesCheckin: async () => [],
    registrarAsistencia: async id => { calls.push(['asistencia', id]); return 'Asistencia registrada' },
    registrarVisita: async (id, nombre) => { calls.push(['visita', id, nombre]); return { id, total_centavos: 5000 } },
    capturarHuella: async () => 'ab'.repeat(400),
    compararHuella: async () => true,
    guardarCliente: async d => { calls.push(['guardar', d]) },
    renovarMembresia: async id => ({id}), ...overrides.api,
  }
  const vue = { ...Vue, onMounted: f => mounted.push(f), onBeforeUnmount: f => unmounted.push(f) }
  const mocks = {
    vue,
    '../services/tauriApi': api,
    './useCamera': { useCamera: () => ({ fotoBase64: Vue.ref('') }) },
    '../services/toast': { mostrarToast: aviso => calls.push(['toast', aviso]) },
    'vue-router': { useRouter: () => ({ push: () => {} }) },
    '@tauri-apps/api/event': { listen: async (name, f) => { listeners.set(name, f); return () => listeners.delete(name) }, ...overrides.events },
    '@tauri-apps/api/tauri': { invoke: async () => ({ estado: 'recuperando', revision: 1, sdk_inicializado: true, captura_comprobada: false }), ...overrides.ipc },
  }
  const datos = new Map()
  const context = createContext({
    navigator: overrides.navigator || {}, crypto: globalThis.crypto, localStorage: overrides.storage || { getItem:k=>datos.get(k)||null, setItem:(k,v)=>datos.set(k,v), removeItem:k=>datos.delete(k) },
    console: { error: (...args) => erroresTecnicos.push(args), warn: () => {}, log: () => {} },
    Date: class extends Date { static now() { return now } },
    setTimeout: (fn, ms) => { const id = ++timerId; timers.set(id, { fn, at: now + ms }); return id },
    clearTimeout: id => timers.delete(id), setInterval: () => ++timerId, clearInterval: () => {},
  })
  const source = new URL(`../src/${file}`, import.meta.url)
  const module = new SourceTextModule(await readFile(source, 'utf8'), { context, identifier: source.href })
  const modules = new Map()
  const linker = async specifier => {
    if (specifier === './useRenovacion') {
      if (!modules.has(specifier)) {
        const child = new SourceTextModule(await readFile(new URL('../src/composables/useRenovacion.js',import.meta.url),'utf8'),{context})
        modules.set(specifier,child); await child.link(linker)
      }
      return modules.get(specifier)
    }
    if (['../services/mensajesLector', '../services/duracionPlanes', '../services/periodosDashboard'].includes(specifier)) {
      const helper = new SourceTextModule(await readFile(new URL(`../src/services/${specifier.split('/').pop()}.js`, import.meta.url), 'utf8'), { context })
      await helper.link(() => { throw Error('El traductor debe ser independiente') })
      return helper
    }
    const values = mocks[specifier]
    if (!values) throw new Error(`Frontera no simulada: ${specifier}`)
    return new SyntheticModule(Object.keys(values), function () {
      for (const [key, value] of Object.entries(values)) this.setExport(key, value)
    }, { context })
  }
  await module.link(linker)
  await module.evaluate()
  return { ns: module.namespace, api, calls, listeners, erroresTecnicos, modules,
    mount: async () => { for (const fn of mounted) await fn() },
    unmount: async () => { for (const fn of unmounted) await fn() },
    advance: ms => { now += ms; for (const [id, timer] of [...timers]) if (timer.at <= now) { timers.delete(id); timer.fn() } },
  }
}

async function registro(overrides) {
  const f = await fixture('composables/useRegistroCliente.js', overrides)
  const state = f.ns.useRegistroCliente()
  await f.mount()
  return { ...f, state }
}

for (const tecnico of [String.raw`SGFPM_OpenDevice: 59; SDK=C:\desarrollo\bin\sgfplib.dll`, 'SGFPM_Init: 6 (DLLLOAD_FAILED_DRV)', 'SQLCipher DPAPI Rust Tauri: error 13', '/opt/private/lib/reader.so']) {
  test(`Captura: el diagnóstico permanece en logs y se presenta un mensaje público seguro (${tecnico.slice(0, 18)})`, async () => {
    const f = await registro({api: {capturarHuella: async () => {throw tecnico}}})
    await f.state.capturarHuellaCliente(1)
    assert.ok(Object.values(MENSAJES_LECTOR).includes(f.state.mensaje.value))
    assert.equal(f.state.mensaje.value.includes(tecnico), false)
    assert.ok(f.erroresTecnicos.some(args => args.includes(tecnico)))
    assert.equal(f.state.template1.value, '')
    assert.equal(detalleLectorPublico({estado:'no_disponible',detalle:tecnico}), MENSAJES_LECTOR.inicio)
    await f.unmount()
  })
}

test('El fallo OpenDevice usa el mensaje público acordado y un reintento puede capturar', async () => {
  assert.equal(mensajeErrorLector('SGFPM_OpenDevice: 59'), 'No se pudo iniciar el lector de huellas. Verifica su conexión USB.')
  let intentos = 0
  const f = await registro({api:{capturarHuella: async () => {if (++intentos===1)throw 'SGFPM_OpenDevice: 59';return 'ab'.repeat(400)}}})
  await f.state.capturarHuellaCliente(1)
  assert.equal(f.state.tipoMensaje.value, 'error')
  await f.state.capturarHuellaCliente(1)
  assert.equal(f.state.template1.value.length, 800)
  assert.equal(f.state.tipoMensaje.value, 'success')
  await f.unmount()
})

test('El alta conserva el rechazo de huella duplicada pero no muestra errores nativos de comparación', async () => {
  for (const error of ['La huella ya pertenece al cliente #7; revisa su registro antes de darlo de alta', String.raw`SGFPM_MatchTemplate: 106; C:\privado\sgfplib.dll`]) {
    const f = await registro({api:{guardarCliente: async () => {throw error}}})
    completar(f.state); await f.state.guardar()
    assert.equal(f.state.tipoMensaje.value, 'error')
    if(error.startsWith('La huella')) assert.equal(f.state.mensaje.value,error)
    else assert.ok(!/SGFPM|106|privado|\.dll/.test(f.state.mensaje.value))
    await f.unmount()
  }
})

test('Check-in oculta los detalles técnicos tanto en lecturas como en búsquedas', async () => {
  const tecnico = String.raw`SGFPM_OpenDevice: 59; C:\privado\sgfplib.dll; SQLCipher DPAPI`
  const f = await fixture('composables/useCheckIn.js', {api:{buscarClientesCheckin:async()=>{throw tecnico}}})
  const s = f.ns.useCheckIn(); await f.mount()
  s.consultaManual.value = 'Alex'; await s.buscarManual()
  assert.equal(s.mensajeManual.value,'No se pudo buscar al cliente. Intenta nuevamente.')
  assert.ok(f.erroresTecnicos.some(args=>args.includes(tecnico)))
  await f.listeners.get('logym://checkin')({payload:{cliente:{id:1,nombre:'Sintético'},resultado:'error: '+tecnico}})
  assert.ok(!/SGFPM|59|privado|\.dll|SQLCipher|DPAPI/.test(s.mensaje.value))
  await s.cerrarModal(); await f.unmount()
})

test('Alta sin huella no captura ni compara y conserva los cobros del formulario', async () => {
  const { state, calls } = await registro({ api: {
    capturarHuella: async () => { throw Error('SDK no debe usarse') },
    compararHuella: async () => { throw Error('SDK no debe usarse') },
  } })
  assert.equal(state.sinHuella.value, false)
  completar(state)
  state.sinHuella.value = true
  await Vue.nextTick()
  assert.equal(state.template1.value, '')
  assert.equal(state.template2.value, '')
  await state.capturarHuellaCliente(1)
  await state.guardar()
  const datos = calls.find(c => c[0] === 'guardar')[1]
  assert.equal(datos.sinHuella, true)
  assert.equal(datos.templateHuella, '')
  assert.equal(datos.tipoMembresiaId, 1)
  assert.equal(datos.incluirInscripcion, true)
  assert.equal(state.tipoMensaje.value, 'success')
})

test('Volver a modalidad biométrica exige dos capturas nuevas', async () => {
  const { state, calls } = await registro()
  completar(state)
  state.sinHuella.value = true
  await Vue.nextTick()
  state.sinHuella.value = false
  await Vue.nextTick()
  await state.guardar()
  assert.equal(calls.filter(c => c[0] === 'guardar').length, 0)
  await state.capturarHuellaCliente(1)
  await state.capturarHuellaCliente(2)
  await state.guardar()
  assert.equal(calls.find(c => c[0] === 'guardar')[1].sinHuella, false)
})

test('Alta sin huella mantiene validación de nombre, teléfonos, fechas y plan', async () => {
  const { state, calls } = await registro()
  state.sinHuella.value = true
  await Vue.nextTick()
  for (const [campo, valor] of [['nombre',''], ['telefono','123'], ['fechaInicioMembresia',''], ['fechaFinMembresia','2020-01-01'], ['tipoMembresiaId','']]) {
    completar(state)
    state.telefono.value = ''
    state.tipoMembresiaId.value = '1'
    state[campo].value = valor
    await state.guardar()
    assert.equal(calls.filter(c => c[0] === 'guardar').length, 0)
  }
})

test('Asistencia manual exige selección explícita y confirmación; evita doble clic', async () => {
  const clientes = [{ id: 7, nombre: 'Alex', telefono: '5511111111' }, { id: 8, nombre: 'Alex', telefono: '5522222222' }]
  let finish, saves = 0, savedId, savedArgs
  const f = await fixture('composables/useCheckIn.js', { api: {
    buscarClientesCheckin: async () => clientes,
    registrarAsistencia: async (...args) => { saves++; savedArgs = args; savedId = args[0]; await new Promise(resolve => { finish = resolve }); return 'Asistencia registrada' },
  } })
  const s = f.ns.useCheckIn(); await f.mount()
  await s.confirmarManual()
  s.consultaManual.value = 'Alex'
  await s.buscarManual()
  assert.equal(s.clienteManual.value, null)
  await s.confirmarManual()
  assert.equal(saves, 0)
  s.seleccionarManual(s.resultadosManual.value[1])
  assert.equal(saves, 0)
  assert.equal(s.modalAbierta.value, true)
  assert.equal(s.modalManual.value, true)
  const pending = s.confirmarManual()
  await s.confirmarManual()
  assert.equal(saves, 1)
  assert.equal(savedId, 8)
  assert.deepEqual(savedArgs, [8], 'Envía únicamente el ID del cliente, sin contraseña')
  finish(); await pending
  assert.equal(s.asistenciaManualRegistrada.value, true)
  assert.match(s.mensaje.value, /ID #8/)
  f.advance(2000); await Promise.resolve()
  assert.equal(s.clienteManual.value, null)
  assert.equal(s.modalAbierta.value, false)
  assert.equal(s.busquedaAbierta.value, false)
  assert.equal(s.consultaManual.value, '')
  assert.equal(f.calls.filter(c => c[0] === 'pausa').length, 0, 'La búsqueda y confirmación no pausan el SDK')
  await f.unmount()
})

test('Cambiar búsqueda invalida selección y descarta respuestas anteriores', async () => {
  let finish
  const f = await fixture('composables/useCheckIn.js', { api: { buscarClientesCheckin: async () => new Promise(resolve => { finish = resolve }) } })
  const s = f.ns.useCheckIn(); await f.mount()
  s.consultaManual.value = 'Alex'
  const pending = s.buscarManual()
  s.consultaManual.value = 'Otro'
  finish([{id: 7, nombre: 'Alex'}]); await pending
  assert.equal(s.resultadosManual.value.length, 0)
  assert.equal(s.clienteManual.value, null)
  assert.equal(s.buscandoManual.value, false)
  await f.unmount()
})

test('Cancelar una búsqueda pendiente vuelve al lector sin tocar el supervisor', async () => {
  let finish
  const f = await fixture('composables/useCheckIn.js', { api: { buscarClientesCheckin: async () => new Promise(resolve => { finish = resolve }) } })
  const s = f.ns.useCheckIn(); await f.mount()
  s.consultaManual.value = 'Alex'
  assert.equal(s.busquedaAbierta.value, false, 'Escribir no sustituye la tarjeta')
  const pending = s.buscarManual()
  assert.equal(s.busquedaAbierta.value, true)
  s.cancelarBusqueda()
  finish([{id: 1}]); await pending
  assert.equal(s.busquedaAbierta.value, false)
  assert.equal(s.resultadosManual.value.length, 0)
  assert.equal(f.calls.length, 0)
  await f.unmount()
})

test('Huella durante búsqueda abre su resultado y conserva la consulta al cerrar', async () => {
  const f = await fixture('composables/useCheckIn.js', { api: { buscarClientesCheckin: async () => [{id: 1, nombre: 'Alex'}] } })
  const s = f.ns.useCheckIn(); await f.mount()
  s.consultaManual.value = 'Alex'; await s.buscarManual()
  await f.listeners.get('logym://checkin')({payload: {cliente: {id: 2, nombre: 'Biométrico'}, resultado: 'registrada'}})
  assert.equal(s.clienteDetectado.value.id, 2)
  assert.equal(s.modalManual.value, false)
  assert.equal(s.estadoCheckin.value, 'success')
  await s.cerrarModal()
  assert.equal(s.busquedaAbierta.value, true)
  assert.equal(s.resultadosManual.value[0].id, 1)
  assert.equal(s.consultaManual.value, 'Alex')
  assert.equal(f.calls.filter(c => c[0] === 'asistencia').length, 0, 'No vuelve a registrar la huella')
  s.cancelarBusqueda(); await f.unmount()
})

test('Huella durante confirmación manual queda en cola sin cambiar la identidad ni perder eventos', async () => {
  let finish, renewals = 0
  const f = await fixture('composables/useCheckIn.js', { api: {
    buscarClientesCheckin: async () => [{id: 1, nombre: 'Manual', estado_membresia: 'vencida'}],
    registrarAsistencia: async () => new Promise(resolve => { finish = resolve }),
    renovarMembresia: async () => { renewals++ },
  } })
  const s = f.ns.useCheckIn(); await f.mount()
  s.consultaManual.value = 'Manual'; await s.buscarManual()
  s.seleccionarManual(s.resultadosManual.value[0])
  await s.solicitarRenovacion()
  const renovacion=f.modules.get('./useRenovacion').namespace.useRenovacion()
  assert.equal(renovacion.abierta.value,true);renovacion.cerrar()
  assert.equal(renewals, 0)
  const pending = s.confirmarManual()
  const recibir = f.listeners.get('logym://checkin')
  await recibir({payload: {cliente: {id: 2, nombre: 'Huella A'}, resultado: 'registrada'}})
  await recibir({payload: {cliente: {id: 3, nombre: 'Huella B'}, resultado: 'duplicada'}})
  assert.equal(s.clienteDetectado.value.id, 1)
  assert.equal(s.lecturasPendientes.value, 2)
  await s.cerrarModal()
  assert.equal(s.modalAbierta.value, true, 'No cierra una operación en curso')
  finish('Asistencia registrada'); await pending
  assert.equal(s.clienteDetectado.value.id, 1, 'Presenta el éxito manual antes de las lecturas pendientes')
  f.advance(2000); await Promise.resolve()
  assert.equal(s.clienteDetectado.value.id, 2)
  assert.equal(s.estadoCheckin.value, 'success')
  assert.equal(s.lecturasPendientes.value, 1)
  await s.cerrarModal()
  assert.equal(s.clienteDetectado.value.id, 3)
  assert.equal(s.estadoCheckin.value, 'warning')
  assert.equal(s.lecturasPendientes.value, 0)
  await s.cerrarModal()
  assert.equal(s.busquedaAbierta.value, false)
  assert.equal(s.modalAbierta.value, false)
  assert.equal(f.calls.filter(c => c[0] === 'pausa').length, 2, 'Solo pausa por evento y libera después de mostrar toda la cola')
  await f.unmount()
})

test('Salir durante confirmación manual impide cierres tardíos', async () => {
  let finish
  const f = await fixture('composables/useCheckIn.js', { api: {
    buscarClientesCheckin: async () => [{id: 1}],
    registrarAsistencia: async () => new Promise(resolve => { finish = resolve }),
  } })
  const s = f.ns.useCheckIn(); await f.mount()
  s.consultaManual.value = 'Cliente'; await s.buscarManual()
  s.seleccionarManual(s.resultadosManual.value[0])
  const pending = s.confirmarManual()
  await f.unmount()
  finish('Asistencia registrada'); await pending
  f.advance(5000)
  assert.equal(s.asistenciaManualRegistrada.value, false)
  assert.equal(f.calls.length, 0)
})

for (const navigator of [{}, { mediaDevices: { getUserMedia: async () => { const e = Error('Denegado'); e.name = 'NotAllowedError'; throw e } } }]) {
  test(`Cámara ${navigator.mediaDevices ? 'sin permiso' : 'ausente'} no genera fotografías ni lanza un error al formulario`, async () => {
    const f = await fixture('composables/useCamera.js', {navigator})
    const mensajes = []
    const camara = f.ns.useCamera((texto, tipo) => mensajes.push([texto, tipo]))
    await camara.iniciarCamara()
    assert.equal(camara.fotoBase64.value, '')
    assert.equal(camara.camaraActiva.value, false)
    assert.equal(mensajes.at(-1)[1], 'error')
    await f.unmount()
  })
}

for (const sinHuella of [false, true]) {
  test(`Guardar cliente ${sinHuella ? 'sin' : 'con'} huella admite fotografía vacía`, async () => {
    const f = await registro()
    completar(f.state); f.state.sinHuella.value = sinHuella
    await f.state.guardar()
    assert.equal(f.calls.find(c => c[0] === 'guardar')[1].fotoBase64, '')
    assert.equal(f.state.tipoMensaje.value, 'success')
    await f.unmount()
  })
}

for (const error of ['Membresía vencida', 'Membresía pendiente', 'Cliente desactivado', 'Ya registró asistencia hoy']) {
  test(`Asistencia manual muestra rechazo del backend: ${error}`, async () => {
    const f = await fixture('composables/useCheckIn.js', { api: {
      buscarClientesCheckin: async () => [{id: 1, nombre: 'Cliente'}],
      registrarAsistencia: async () => { throw Error(error) },
    } })
    const s = f.ns.useCheckIn(); await f.mount()
    s.consultaManual.value = 'Cliente'; await s.buscarManual()
    s.seleccionarManual(s.resultadosManual.value[0]); await s.confirmarManual()
    assert.ok(s.mensaje.value.includes(error))
    assert.equal(s.modalAbierta.value, true)
    assert.equal(s.estadoCheckin.value, 'error')
    assert.equal(s.registrandoManual.value, false)
    await s.cerrarModal()
    assert.equal(s.busquedaAbierta.value, false)
    await f.unmount()
  })
}
function completar(state) {
  state.nombre.value = 'Cliente sintético'
  state.fechaRegistro.value = '2024-01-31'
  state.fechaInicioMembresia.value = '2024-01-31'
  state.fechaFinMembresia.value = '2024-02-29'
  state.template1.value = 'ab'.repeat(400)
  state.template2.value = 'ab'.repeat(400)
}

for (const [inicio, meses, fin] of [
  ['2023-01-31', 1, '2023-02-28'], ['2024-01-31', 1, '2024-02-29'],
  ['2024-03-31', 1, '2024-04-30'], ['2024-12-31', 1, '2025-01-31'],
  ['2024-02-29', 12, '2025-02-28'], ['2000-01-31', 1, '2000-02-29'],
]) test(`Registro: calendario ${inicio} + ${meses} meses`, async () => {
  const { state } = await registro()
  state.tiposMembresia.value = [{ id: 1, duracion_meses: meses }]
  state.fechaInicioMembresia.value = inicio
  await Vue.nextTick()
  assert.equal(state.fechaFinMembresia.value, fin)
})

for (const value of [null, '', 'ab', 'zz'.repeat(400), 'ab'.repeat(401)]) {
  test(`Captura inválida: ${value === null ? 'timeout/null' : `longitud ${value.length}`}`, async () => {
    const { state } = await registro({ api: { capturarHuella: async () => value } })
    await state.capturarHuellaCliente(1)
    assert.equal(state.template1.value, '')
    assert.equal(state.capturandoHuella.value, false)
    assert.equal(state.tipoMensaje.value, 'error')
  })
}

test('Dos capturas coincidentes y nueva primera captura invalida confirmación', async () => {
  const { state } = await registro()
  await state.capturarHuellaCliente(2)
  assert.equal(state.template2.value, '')
  await state.capturarHuellaCliente(1)
  await state.capturarHuellaCliente(2)
  assert.equal(state.template2.value.length, 800)
  await state.capturarHuellaCliente(1)
  assert.equal(state.template2.value, '')
})

test('Comparación 1:1 negativa impide confirmar y guardar', async () => {
  const { state, calls } = await registro({ api: { compararHuella: async () => false } })
  completar(state)
  await state.capturarHuellaCliente(2)
  assert.equal(state.template2.value, '')
  completar(state)
  await state.guardar()
  assert.equal(calls.filter(c => c[0] === 'guardar').length, 0)
  assert.equal(state.guardando.value, false)
})

test('Entradas inválidas impiden enviar IPC de alta', async () => {
  const { state, calls } = await registro()
  for (const [field, value] of [['nombre', ''], ['nombre', 'x'.repeat(121)], ['nombre', 'a\nb'], ['telefono', '123'], ['telefonoEmergencia', 'abcdefghij'], ['fechaFinMembresia', '2020-01-01'], ['tipoMembresiaId', '']]) {
    completar(state); state.telefono.value = ''; state.telefonoEmergencia.value = ''; state.tipoMembresiaId.value = '1'
    state[field].value = value
    await state.guardar()
    assert.equal(calls.filter(c => c[0] === 'guardar').length, 0, field)
  }
})

test('Doble clic mientras guarda envía una sola operación', async () => {
  let finish
  const pending = new Promise(resolve => { finish = resolve })
  let saves = 0
  const { state } = await registro({ api: { guardarCliente: async () => { saves++; await pending } } })
  completar(state)
  const first = state.guardar()
  await Promise.resolve(); await Promise.resolve()
  await state.guardar()
  finish(); await first
  assert.equal(saves, 1)
  assert.equal(state.guardando.value, false)
})

test('Error de persistencia conserva formulario y permite reintento', async () => {
  const { state } = await registro({ api: { guardarCliente: async () => { throw Error('fallo simulado') } } })
  completar(state)
  await state.guardar()
  assert.equal(state.nombre.value, 'Cliente sintético')
  assert.equal(state.template1.value.length, 800)
  assert.equal(state.tipoMensaje.value, 'error')
  assert.equal(state.guardando.value, false)
})

test('Registro pausa el worker al montar y lo libera al salir', async () => {
  const f = await registro()
  assert.equal(f.calls[0][0], 'pausa'); assert.equal(f.calls[0][1], true)
  await f.unmount()
  assert.equal(f.calls.at(-1)[1], false)
})

test('Registro: errores de captura y guardas de formulario liberan flags', async () => {
  const { state, calls } = await registro({ api: { capturarHuella: async () => { throw Error('SDK simulado') } } })
  await state.capturarHuellaCliente(1)
  assert.equal(state.capturandoHuella.value, false)
  assert.equal(state.tipoMensaje.value, 'error')
  completar(state); state.capturandoHuella.value = true
  await state.guardar(); await state.capturarHuellaCliente(1)
  state.capturandoHuella.value = false; state.fechaInicioMembresia.value = ''
  await state.guardar()
  assert.equal(calls.filter(c => c[0] === 'guardar').length, 0)
})

test('Registro: catálogo alternativo, vacío y error de inicialización', async () => {
  const alternate = await registro({ api: { listarTiposMembresia: async () => [{ id: 5, nombre: 'Semestral', duracion_meses: 6 }] } })
  assert.equal(alternate.state.tipoMembresiaId.value, '5')
  const empty = await registro({ api: { listarTiposMembresia: async () => [] } })
  assert.equal(empty.state.tipoMembresiaId.value, '')
  const failure = await registro({ api: { inicializarBaseDeDatos: async () => { throw Error('SQLCipher simulado') } } })
  assert.equal(failure.state.tipoMensaje.value, 'error')
})

test('CheckIn: presencia/SDK inicializado no equivale a captura comprobada', async () => {
  const f = await fixture('composables/useCheckIn.js')
  const state = f.ns.useCheckIn(); await f.mount()
  assert.equal(state.lectorActivo.value, false)
  const update = f.listeners.get('logym://lector-estado')
  update({ payload: { estado: 'disponible', revision: 2, captura_comprobada: true } })
  assert.equal(state.lectorActivo.value, true)
  update({ payload: { estado: 'no_disponible', revision: 1 } })
  assert.equal(state.lectorActivo.value, true) // Evento viejo ignorado.
  update({ payload: { estado: 'no_disponible', revision: 3 } })
  assert.equal(state.lectorActivo.value, false)
  await f.unmount(); assert.equal(f.listeners.size, 0)
})

test('CheckIn: no ofrece plan exclusivo de alta para renovación', async () => {
  const f = await fixture('composables/useCheckIn.js', { api: { listarTiposMembresia: async () => [
    { id: 1, solo_alta: false }, { id: 2, solo_alta: true },
  ] } })
  const state = f.ns.useCheckIn(); await f.mount()
  state.clienteDetectado.value={id:1};await state.solicitarRenovacion()
  const renovacion=f.modules.get('./useRenovacion').namespace.useRenovacion()
  assert.equal(renovacion.planes.value.length, 1)
  assert.equal(renovacion.planes.value[0].id, 1)
  await f.unmount()
})

for (const [resultado, estado] of [['registrada', 'success'], ['duplicada', 'warning'], ['membresia_pendiente', 'error'], ['membresia_vencida', 'error'], ['error: fallo simulado', 'error']]) {
  test(`CheckIn: resultado ${resultado}, pausa y cierre de modal`, async () => {
    const f = await fixture('composables/useCheckIn.js')
    const state = f.ns.useCheckIn(); await f.mount()
    const recibir = f.listeners.get('logym://checkin')
    await recibir({ payload: {} }); assert.equal(state.modalAbierta.value, false)
    await recibir({ payload: { cliente: { id: 1, nombre: 'Sintético', fecha_inicio_membresia: '2024-01-01' }, resultado } })
    assert.equal(state.modalAbierta.value, true)
    assert.equal(state.estadoCheckin.value, estado)
    assert.equal(f.calls.at(-1)[1], true)
    await state.capturarAcceso(); assert.match(state.mensaje.value, /lector/)
    await state.cerrarModal()
    assert.equal(state.modalAbierta.value, false)
    assert.equal(state.clienteDetectado.value, null)
    assert.equal(f.calls.at(-1)[1], false)
    await f.unmount()
  })
}

test('CheckIn: formato y etiquetas de todos los estados de membresía', async () => {
  const f = await fixture('composables/useCheckIn.js'); const state = f.ns.useCheckIn()
  assert.equal(state.formatearFecha(''), 'Sin fecha')
  assert.equal(state.formatearFecha('2024-02-29'), '29/02/2024')
  assert.equal(state.formatearFecha('invalida'), 'invalida')
  for (const estado of ['pendiente', 'vencida', 'por_vencer', 'activa', 'sin_fecha']) {
    assert.equal(typeof state.etiquetaEstado(estado), 'string')
    assert.equal(typeof state.diasRestantes({ estado_membresia: estado, dias_restantes: -3, fecha_inicio_membresia: '2024-02-29', fecha_fin_membresia: '2024-03-29' }), 'string')
  }
})

test('CheckIn: renovación confirmada, doble clic y liberación al desmontar', async () => {
  let finish, count = 0
  const pending = new Promise(resolve => { finish = resolve })
  const f = await fixture('composables/useCheckIn.js', { api: { renovarMembresia: async () => { count++; await pending; return { id:1,nombre: 'Sintético', fecha_fin_membresia: '2024-12-31' } } } })
  const state = f.ns.useCheckIn(); await f.mount()
  assert.equal(count, 0)
  await f.listeners.get('logym://checkin')({ payload: { cliente: { id: 1, nombre: 'Sintético' }, resultado: 'duplicada' } })
  await state.solicitarRenovacion()
  const modal=f.modules.get('./useRenovacion').namespace.useRenovacion()
  assert.equal(modal.abierta.value,true)
  const first = modal.confirmar(); await modal.confirmar(); assert.equal(count, 1)
  finish(); await first
  await Vue.nextTick()
  assert.equal(state.clienteDetectado.value.fecha_fin_membresia,'2024-12-31')
  assert.equal(modal.abierta.value,false);assert.equal(modal.guardando.value,false)
  await f.unmount(); assert.equal(f.calls.at(-1)[1], false)
})

test('CheckIn: fallos de consulta y listener; el catálogo se consulta en el modal común', async () => {
  const f = await fixture('composables/useCheckIn.js', {
    api: { listarTiposMembresia: async () => { throw Error('catálogo simulado') }, renovarMembresia: async () => { throw Error('pago simulado') } },
    ipc: { invoke: async () => { throw Error('IPC simulado') } },
    events: { listen: async () => { throw Error('listener simulado') } },
  })
  const state = f.ns.useCheckIn(); await f.mount()
  assert.equal(state.lectorActivo.value, false); assert.equal(state.estadoLector.value, 'no_disponible')
  assert.match(state.mensaje.value, /lector/)
  state.clienteDetectado.value = { id: 1 }
  await state.solicitarRenovacion()
  const modal=f.modules.get('./useRenovacion').namespace.useRenovacion()
  assert.match(modal.error.value,/No se pudieron cargar/)
  assert.ok(!modal.error.value.includes('catálogo simulado'))
  assert.ok(f.erroresTecnicos.some(args => args.some(a => String(a).includes('catálogo simulado'))))
  await f.unmount()
})

test('Toasts: límite, cierre, pausa y reanudación sin tiempos reales', async () => {
  const f = await fixture('services/toast.js')
  const { avisos } = f.ns.useToasts()
  const ids = []
  for (let i = 0; i < 5; i++) ids.push(f.ns.mostrarToast({ titulo: `Aviso ${i}`, duracion: 1000 }))
  assert.equal(avisos.value.length, 4)
  assert.equal(avisos.value.some(a => a.id === ids[0]), false)
  f.advance(400); f.ns.pausarToast(ids[4]); f.advance(1000)
  assert.equal(avisos.value.length, 1)
  f.ns.reanudarToast(ids[4]); f.advance(599); assert.equal(avisos.value.length, 1)
  f.advance(1); assert.equal(avisos.value.length, 0)
  const id = f.ns.mostrarToast({ titulo: 'Persistente', duracion: 0 })
  f.advance(100000); assert.equal(avisos.value.length, 1)
  f.ns.cerrarToast(id); assert.equal(avisos.value.length, 0)
})
