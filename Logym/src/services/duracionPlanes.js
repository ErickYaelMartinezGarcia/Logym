// Mensual encabeza todos los catálogos; el resto conserva su orden original.
export function ordenarPlanes(planes) {
  const esMensual = plan => plan.nombre?.trim().toLocaleLowerCase('es-MX') === 'mensual'
  return [...planes].sort((a, b) => Number(esMensual(b)) - Number(esMensual(a)))
}

export function duracionPlan(plan) {
  const dias = plan.duracion_dias
  if (dias != null) return `${dias} ${dias === 1 ? 'día' : 'días'}`
  return `${plan.duracion_meses} ${plan.duracion_meses === 1 ? 'mes' : 'meses'}`
}

export function fechaFinPlan(fecha, plan) {
  const inicio = new Date(`${fecha}T12:00:00`)
  if (Number.isNaN(inicio.getTime())) return ''
  if (plan.duracion_dias != null) {
    if (!Number.isInteger(plan.duracion_dias) || plan.duracion_dias < 1 || plan.duracion_dias > 3660) return ''
    inicio.setDate(inicio.getDate() + plan.duracion_dias - 1)
  } else {
    const dia = inicio.getDate()
    inicio.setDate(1)
    inicio.setMonth(inicio.getMonth() + plan.duracion_meses)
    inicio.setDate(Math.min(dia, new Date(inicio.getFullYear(), inicio.getMonth() + 1, 0).getDate()))
  }
  return `${inicio.getFullYear()}-${String(inicio.getMonth() + 1).padStart(2, '0')}-${String(inicio.getDate()).padStart(2, '0')}`
}
