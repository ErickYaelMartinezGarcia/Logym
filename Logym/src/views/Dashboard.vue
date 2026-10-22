<script setup>
import { onMounted, ref } from 'vue'
import { inicializarBaseDeDatos, listarClientes, obtenerMetricasDashboard } from '../db'

const clientes = ref([])
const metricas = ref([
  { titulo: 'Clientes registrados', valor: '0' },
  { titulo: 'Asistencias hoy', valor: '0' },
  { titulo: 'Nuevos esta semana', valor: '0' },
  { titulo: 'Huellas activas', valor: '0' }
])

/**
 * Carga clientes y métricas sencillas para el dashboard.
 */
onMounted(async () => {
  try {
    await inicializarBaseDeDatos()
    clientes.value = await listarClientes()
    const datos = await obtenerMetricasDashboard()

    metricas.value[0].valor = String(datos.total_clientes || 0)
    metricas.value[1].valor = String(datos.asistencias_hoy || 0)
    metricas.value[2].valor = String(datos.nuevos_semana || 0)
    metricas.value[3].valor = String(datos.huella_activas || 0)
  } catch (error) {
    console.error('Error al cargar dashboard:', error)
  }
})
</script>

<template>
  <div class="min-h-screen flex bg-[#0A0A0A] text-white">
    <aside class="w-72 p-6 border-r border-[#1F1F1F] bg-[#111111]">
      <div class="bg-[#141414] border border-[#1F1F1F] rounded-[24px] shadow-2xl backdrop-blur-xl p-6 space-y-3">
        <h2 class="text-2xl text-[#C41E3A] font-semibold">GYM OS</h2>
        <p class="text-gray-300">Panel principal</p>
        <nav class="space-y-2">
          <a class="block rounded-xl px-3 py-2 bg-[#8B0000]" href="#/">Check-In</a>
          <a class="block rounded-xl px-3 py-2 bg-[#1A1A1A] border border-[#1F1F1F]" href="#/registro">Registro</a>
          <a class="block rounded-xl px-3 py-2 bg-[#1A1A1A] border border-[#1F1F1F]" href="#/dashboard">Dashboard</a>
        </nav>
      </div>
    </aside>

    <main class="flex-1 p-8 space-y-8">
      <section class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4">
        <div v-for="item in metricas" :key="item.titulo" class="bg-[#141414] border border-[#1F1F1F] rounded-[24px] shadow-2xl backdrop-blur-xl p-6">
          <p class="text-sm text-gray-400">{{ item.titulo }}</p>
          <p class="text-3xl font-bold text-[#C41E3A] mt-2">{{ item.valor }}</p>
        </div>
      </section>

      <section class="bg-[#141414] border border-[#1F1F1F] rounded-[24px] shadow-2xl backdrop-blur-xl p-6">
        <h3 class="text-xl font-semibold mb-4">Clientes recientes</h3>
        <div class="overflow-x-auto">
          <table class="w-full text-left">
            <thead>
              <tr class="text-gray-400 border-b border-[#1F1F1F]">
                <th class="py-3">ID</th>
                <th class="py-3">Nombre</th>
                <th class="py-3">Teléfono</th>
                <th class="py-3">Registro</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="cliente in clientes" :key="cliente.id" class="border-b border-[#1F1F1F]">
                <td class="py-3">{{ cliente.id }}</td>
                <td class="py-3">{{ cliente.nombre }}</td>
                <td class="py-3">{{ cliente.telefono || 'Sin teléfono' }}</td>
                <td class="py-3">{{ cliente.fecha_registro }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section class="bg-[#141414] border border-[#1F1F1F] rounded-[24px] shadow-2xl backdrop-blur-xl p-6">
        <h3 class="text-xl font-semibold mb-4">Asistencias semanales</h3>
        <div class="grid grid-cols-7 gap-2 h-44 items-end">
          <div class="bg-[#8B0000] rounded-t-xl" style="height: 35%"></div>
          <div class="bg-[#8B0000] rounded-t-xl" style="height: 50%"></div>
          <div class="bg-[#8B0000] rounded-t-xl" style="height: 45%"></div>
          <div class="bg-[#8B0000] rounded-t-xl" style="height: 70%"></div>
          <div class="bg-[#8B0000] rounded-t-xl" style="height: 60%"></div>
          <div class="bg-[#8B0000] rounded-t-xl" style="height: 80%"></div>
          <div class="bg-[#8B0000] rounded-t-xl" style="height: 55%"></div>
        </div>
      </section>
    </main>
  </div>
</template>
