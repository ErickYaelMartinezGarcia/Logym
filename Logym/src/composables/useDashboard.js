import { computed, onMounted, ref } from 'vue'
import {
  inicializarBaseDeDatos,
  listarClientes,
  listarClientesMembresia,
  obtenerMetricasDashboard
} from '../services/tauriApi'

export function useDashboard() {
  const clientes = ref([])
  const clientesMembresia = ref([])
  const busqueda = ref('')
  const resumenAsistencias = ref({
    ultimos7Dias: '0',
    diaMasConcurrido: 'Sin datos',
    horaMayorAfluencia: 'Sin datos'
  })
  const metricas = ref([
    { titulo: 'Miembros activos', valor: '0', trend: '+8.4%', icon: '◉' },
    { titulo: 'Entradas de hoy', valor: '0', trend: 'En tiempo real', icon: '↗' },
    { titulo: 'Nuevos esta semana', valor: '0', trend: 'Últimos 7 días', icon: '+' },
    { titulo: 'Huellas activas', valor: '0', trend: 'Biometría lista', icon: '⌁' }
  ])

  const clientesFiltrados = computed(() => {
    const termino = busqueda.value
      .replace(/[\u0000-\u001F\u007F]/g, '')
      .trim()
      .toLowerCase()
      .slice(0, 80)

    if (!termino) return []

    return clientesMembresia.value
      .filter((cliente) => {
        return [cliente.nombre, cliente.telefono, cliente.telefono_emergencia, cliente.id]
          .some((valor) => String(valor || '').toLowerCase().includes(termino))
      })
      .slice(0, 2)
  })

  const alertasMembresia = computed(() => clientesMembresia.value
    .filter(({ estado_membresia }) => ['vencida', 'por_vencer'].includes(estado_membresia)))

  const membresiasVencidas = computed(() => clientesMembresia.value
    .filter(({ estado_membresia }) => estado_membresia === 'vencida').length)
  const membresiasPorVencer = computed(() => clientesMembresia.value
    .filter(({ estado_membresia }) => estado_membresia === 'por_vencer').length)

  async function cargarDashboard() {
    await inicializarBaseDeDatos()
    const [clientesCargados, membresias, datos] = await Promise.all([
      listarClientes(),
      listarClientesMembresia(),
      obtenerMetricasDashboard()
    ])
    clientes.value = clientesCargados
    clientesMembresia.value = membresias
    metricas.value = [
      { titulo: 'Miembros activos', valor: String(datos.total_clientes || 0), trend: '+8.4%', icon: '◉' },
      { titulo: 'Entradas de hoy', valor: String(datos.asistencias_hoy || 0), trend: 'En tiempo real', icon: '↗' },
      { titulo: 'Nuevos esta semana', valor: String(datos.nuevos_semana || 0), trend: 'Últimos 7 días', icon: '+' },
      { titulo: 'Huellas activas', valor: String(datos.huella_activas || 0), trend: 'Biometría lista', icon: '⌁' }
    ]
    resumenAsistencias.value = {
      ultimos7Dias: String(datos.asistencias_ultimos_7_dias || 0),
      diaMasConcurrido: datos.dia_mas_concurrido || 'Sin datos',
      horaMayorAfluencia: datos.hora_mayor_afluencia || 'Sin datos'
    }
  }

  onMounted(() => cargarDashboard().catch((error) => console.error('Error al cargar dashboard:', error)))

  return {
    clientes, busqueda, clientesFiltrados, resumenAsistencias, metricas,
    alertasMembresia, membresiasVencidas, membresiasPorVencer
  }
}
