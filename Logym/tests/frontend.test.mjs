import test from 'node:test'
import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { SourceTextModule, SyntheticModule, createContext } from 'node:vm'
import * as Vue from 'vue'

// Ejecuta los módulos de producción. Se simulan únicamente sus fronteras:
// IPC, cámara, router, lifecycle y reloj; no se replica la lógica de negocio.
async function fixture(file, overrides = {}) {
  const mounted = [], unmounted = [], listeners = new Map(), calls = []
  let now = 0, timerId = 0
  const timers = new Map()
  const api = {
    pausarLectorGlobal: async v => { calls.push(['pausa', v]) },
    inicializarBaseDeDatos: async () => {},
    listarTiposMembresia: async () => [{ id: 1, nombre: 'Mensual', duracion_meses: 1, solo_alta: false }],
    obtenerPrecioInscripcion: async () => 15000,
    capturarHuella: async () => 'ab'.repeat(400),
    compararHuella: async () => true,
    guardarCliente: async d => { calls.push(['guardar', d]) },
    renovarMembresia: async () => ({}), ...overrides.api,
  }
  const vue = { ...Vue, onMounted: f => mounted.push(f), onBeforeUnmount: f => unmounted.push(f) }
  const mocks = {
    vue,
    '../services/tauriApi': api,
    './useCamera': { useCamera: () => ({ fotoBase64: Vue.ref('') }) },
    'vue-router': { useRouter: () => ({ push: () => {} }) },
    '@tauri-apps/api/event': { listen: async (name, f) => { listeners.set(name, f); return () => listeners.delete(name) }, ...overrides.events },
    '@tauri-apps/api/tauri': { invoke: async () => ({ estado: 'recuperando', revision: 1, sdk_inicializado: true, captura_comprobada: false }), ...overrides.ipc },
  }
  const context = createContext({
    console: { error: () => {}, warn: () => {}, log: () => {} },
    Date: class extends Date { static now() { return now } },
    setTimeout: (fn, ms) => { const id = ++timerId; timers.set(id, { fn, at: now + ms }); return id },
    clearTimeout: id => timers.delete(id), setInterval: () => ++timerId, clearInterval: () => {},
  })
  const source = new URL(`../src/${file}`, import.meta.url)
  const module = new SourceTextModule(await readFile(source, 'utf8'), { context, identifier: source.href })
  await module.link(specifier => {
    const values = mocks[specifier]
    if (!values) throw new Error(`Frontera no simulada: ${specifier}`)
    return new SyntheticModule(Object.keys(values), function () {
      for (const [key, value] of Object.entries(values)) this.setExport(key, value)
    }, { context })
  })
  await module.evaluate()
  return { ns: module.namespace, api, calls, listeners,
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
  state.capturandoHuella.value = false; state.fechaRegistro.value = ''
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
  assert.equal(state.tiposMembresia.value.length, 1)
  assert.equal(state.tiposMembresia.value[0].id, 1)
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
  const f = await fixture('composables/useCheckIn.js', { api: { renovarMembresia: async () => { count++; await pending; return { nombre: 'Sintético', fecha_fin_membresia: '2024-12-31' } } } })
  const state = f.ns.useCheckIn(); await f.mount()
  await state.renovar(); assert.equal(count, 0)
  await f.listeners.get('logym://checkin')({ payload: { cliente: { id: 1, nombre: 'Sintético' }, resultado: 'duplicada' } })
  state.solicitarRenovacion(); assert.equal(state.confirmandoRenovacion.value, true)
  const first = state.renovar(); await state.renovar(); assert.equal(count, 1)
  finish(); await first
  assert.equal(state.estadoCheckin.value, 'success')
  assert.equal(state.confirmandoRenovacion.value, false)
  assert.equal(state.procesandoRenovacion.value, false)
  await f.unmount(); assert.equal(f.calls.at(-1)[1], false)
})

test('CheckIn: fallos de consulta, listener, catálogo y renovación se muestran', async () => {
  const f = await fixture('composables/useCheckIn.js', {
    api: { listarTiposMembresia: async () => { throw Error('catálogo simulado') }, renovarMembresia: async () => { throw Error('pago simulado') } },
    ipc: { invoke: async () => { throw Error('IPC simulado') } },
    events: { listen: async () => { throw Error('listener simulado') } },
  })
  const state = f.ns.useCheckIn(); await f.mount()
  assert.equal(state.lectorActivo.value, false); assert.equal(state.estadoLector.value, 'no_disponible')
  assert.match(state.mensaje.value, /lector/)
  state.clienteDetectado.value = { id: 1 }; state.tipoMembresiaId.value = '1'
  await state.renovar()
  assert.equal(state.estadoCheckin.value, 'error')
  assert.equal(state.procesandoRenovacion.value, false)
  assert.match(state.mensaje.value, /pago simulado/)
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
