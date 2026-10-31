import { invoke } from '@tauri-apps/api/tauri'

async function ejecutar(comando, args) {
  try {
    return await invoke(comando, args)
  } catch (error) {
    console.error(`Error en comando Tauri "${comando}":`, error)
    throw error
  }
}

export const inicializarBaseDeDatos = () => ejecutar('inicializar_db')
export const guardarCliente = (datos) => ejecutar('guardar_cliente', { cliente: datos })
export const listarClientes = () => ejecutar('listar_clientes')
export const listarClientesMembresia = () => ejecutar('listar_clientes_membresia')
export const buscarClientePorHuella = () => ejecutar('buscar_cliente_por_huella')
export const checkIn = () => ejecutar('check_in')
export const capturarHuella = () => ejecutar('capturar_huella')
export const registrarAsistencia = (clienteId) => ejecutar('registrar_asistencia', { cliente_id: clienteId })
export const renovarMembresia = (clienteId) => ejecutar('renovar_membresia', {datos: { clienteId }})
export const obtenerMetricasDashboard = () => ejecutar('obtener_metricas')
export const compararHuella = (t1, t2) => ejecutar('comparar_huella', {
  t1: t1,
  t2: t2
})

export const actualizarCliente = (id, datos) => ejecutar('actualizar_cliente', { id, datos })
export const establecerClienteActivo = (id, activo) => ejecutar('establecer_cliente_activo', { id, activo })
export const eliminarClienteDefinitivamente = (id, confirmacion) => ejecutar('eliminar_cliente_definitivamente', { id, confirmacion })
