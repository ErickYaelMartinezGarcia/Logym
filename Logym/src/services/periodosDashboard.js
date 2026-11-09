export const fechaLocal = fecha => [fecha.getFullYear(), String(fecha.getMonth() + 1).padStart(2, '0'), String(fecha.getDate()).padStart(2, '0')].join('-')

export function ventanaSieteDias(conteos, hoy = new Date()) {
  const mapa = new Map(conteos.map(d => [d.fecha, Number(d.total)]))
  return Array.from({ length: 7 }, (_, i) => {
    const fecha = new Date(hoy.getFullYear(), hoy.getMonth(), hoy.getDate() - 6 + i, 12)
    const clave = fechaLocal(fecha)
    return { fecha: clave, etiqueta: fecha.toLocaleDateString('es-MX', { weekday: 'short' }), total: mapa.get(clave) ?? 0 }
  })
}

export function filtrarCriticas(clientes, meses = '3', hoy = new Date()) {
  const limite = new Date(hoy.getFullYear(), hoy.getMonth(), 1, 12)
  limite.setMonth(limite.getMonth() - Number(meses === 'todas' ? 0 : meses))
  limite.setDate(Math.min(hoy.getDate(), new Date(limite.getFullYear(), limite.getMonth() + 1, 0).getDate()))
  const desde = fechaLocal(limite)
  return clientes.filter(c => c.activo && (c.estado_membresia === 'por_vencer' || (c.estado_membresia === 'vencida' && (meses === 'todas' || c.fecha_fin_membresia >= desde))))
}
