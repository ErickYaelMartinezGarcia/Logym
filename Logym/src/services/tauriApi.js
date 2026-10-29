import { invoke } from '@tauri-apps/api/tauri'

const esNavegador = typeof window !== 'undefined' && !window.__TAURI__
const demoTemplate = 'demo-secugen-hamster-20-pro'
const demoFoto = "data:image/svg+xml;charset=UTF-8,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 240 240'%3E%3Crect width='240' height='240' fill='%231a2028'/%3E%3Ccircle cx='120' cy='92' r='48' fill='%23ee2046'/%3E%3Cpath d='M42 220c8-48 38-74 78-74s70 26 78 74' fill='%23ee2046'/%3E%3Ctext x='120' y='106' text-anchor='middle' font-family='Arial' font-size='42' font-weight='700' fill='white'%3EV%3C/text%3E%3C/svg%3E"
let demoAsistenciaRegistrada = false
const demoCliente = () => ({
  id: 1,
  nombre: 'Alex Rivera',
  telefono: '+52 55 0000 0000',
  telefono_emergencia: '',
  foto_base64: demoFoto,
  fecha_registro: new Date(Date.now() - 60 * 86400000).toISOString().slice(0, 10),
  fecha_inicio_membresia: new Date(Date.now() - 60 * 86400000).toISOString().slice(0, 10),
  fecha_fin_membresia: new Date(Date.now() - 1 * 86400000).toISOString().slice(0, 10),
  estado_membresia: 'vencida',
  dias_restantes: -1
})

async function ejecutar(comando, args) {
  if (esNavegador) {
    if (comando === 'inicializar_db') return true
    if (comando === 'capturar_huella') return demoTemplate
    if (comando === 'buscar_cliente_por_huella') return args.template === demoTemplate ? demoCliente() : null
    if (comando === 'registrar_asistencia') {
      const resultado = !demoAsistenciaRegistrada
      demoAsistenciaRegistrada = true
      return resultado
    }
    if (comando === 'renovar_membresia') {
      const fechaInicio = new Date()
      const fechaFin = new Date(fechaInicio.getTime() + 30 * 86400000)
      return {
        ...demoCliente(),
        fecha_inicio_membresia: fechaInicio.toISOString().slice(0, 10),
        fecha_fin_membresia: fechaFin.toISOString().slice(0, 10),
        estado_membresia: 'activa',
        dias_restantes: 30
      }
    }
    if (comando === 'guardar_template') return true
    if (comando === 'listar_clientes') return [demoCliente()]
    if (comando === 'listar_clientes_membresia') return [demoCliente()]
    if (comando === 'obtener_metricas') {
      return {
        total_clientes: 1,
        asistencias_hoy: demoAsistenciaRegistrada ? 1 : 0,
        asistencias_ultimos_7_dias: demoAsistenciaRegistrada ? 1 : 0,
        dia_mas_concurrido: 'Hoy',
        hora_mayor_afluencia: '08:00',
        nuevos_semana: 1,
        huella_activas: 1
      }
    }
  }

  try {
    return await invoke(comando, args)
  } catch (error) {
    console.error(`Error en comando Tauri "${comando}":`, error)
    throw error
  }
}

export const inicializarBaseDeDatos = () => ejecutar('inicializar_db')
export const guardarCliente = (datos) => ejecutar('guardar_cliente', { datos })
export const listarClientes = () => ejecutar('listar_clientes')
export const listarClientesMembresia = () => ejecutar('listar_clientes_membresia')
export const buscarClientePorHuella = (template) => ejecutar('buscar_cliente_por_huella', { template })
export const registrarAsistencia = (clienteId) => ejecutar('registrar_asistencia', { cliente_id: clienteId })
export const renovarMembresia = (clienteId) => ejecutar('renovar_membresia', {
  datos: { clienteId }
})
export const obtenerMetricasDashboard = () => ejecutar('obtener_metricas')
export const capturarHuella = () => ejecutar('capturar_huella')
export const guardarTemplate = (clienteId, template) => ejecutar('guardar_template', {
  cliente_id: String(clienteId),
  template
})
