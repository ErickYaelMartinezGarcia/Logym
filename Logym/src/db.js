// Compatibilidad temporal para módulos antiguos. La API vive en services/tauriApi.js.
export {
  inicializarBaseDeDatos,
  guardarCliente as guardarClienteEnBaseDeDatos,
  listarClientesMembresia,
  buscarClientePorHuella,
  registrarAsistencia,
  renovarMembresia as renovarMembresiaCliente,
  listarClientes,
  obtenerMetricasDashboard,
  capturarHuella
} from './services/tauriApi'
