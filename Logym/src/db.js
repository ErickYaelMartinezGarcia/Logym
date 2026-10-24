import { invoke } from '@tauri-apps/api/tauri'

/**
 * Crea el archivo gym.db y las tablas base del sistema.
 */
export async function inicializarBaseDeDatos() {
  try {
    await invoke('inicializar_db')
  } catch (error) {
    console.error('Error al inicializar gym.db:', error)
    throw error
  }
}

/**
 * Guarda un cliente nuevo y regresa su id.
 */
export async function guardarClienteEnBaseDeDatos(datosCliente) {
  try {
    const id = await invoke('guardar_cliente', { datos: datosCliente })
    return id
  } catch (error) {
    console.error('Error al guardar cliente en db:', error)
    throw error
  }
}

/**
 * Regresa clientes con fechas de membresía y estado de vencimiento.
 */
export async function listarClientesMembresia() {
  try {
    return await invoke('listar_clientes_membresia')
  } catch (error) {
    console.error('Error al listar clientes con membresía:', error)
    throw error
  }
}

/**
 * Busca un cliente por el template de huella capturado.
 */
export async function buscarClientePorHuella(template) {
  try {
    return await invoke('buscar_cliente_por_huella', { template })
  } catch (error) {
    console.error('Error al buscar cliente por huella:', error)
    throw error
  }
}

/**
 * Renueva membresía y devuelve el estado actualizado del cliente.
 */
export async function renovarMembresiaCliente(clienteId, dias, monto) {
  try {
    return await invoke('renovar_membresia', {
      datos: { clienteId, dias, monto }
    })
  } catch (error) {
    console.error('Error al renovar membresía:', error)
    throw error
  }
}

/**
 * Regresa la lista de clientes para la tabla del dashboard.
 */
export async function listarClientes() {
  try {
    return await invoke('listar_clientes')
  } catch (error) {
    console.error('Error al listar clientes:', error)
    throw error
  }
}

/**
 * Regresa métricas sencillas para cards del dashboard.
 */
export async function obtenerMetricasDashboard() {
  try {
    return await invoke('obtener_metricas')
  } catch (error) {
    console.error('Error al cargar métricas:', error)
    throw error
  }
}
