export const MENSAJES_LECTOR = Object.freeze({
  inicio: 'No se pudo iniciar el lector de huellas. Verifica su conexión USB.',
  captura: 'No se pudo leer la huella. Coloca el dedo completo e intenta nuevamente.',
  verificacion: 'No se pudo verificar la huella. Intenta nuevamente.',
  comprobando: 'Comprobando el lector de huellas.',
  esperando: 'Coloca el dedo para comprobar que el lector está listo.',
  disponible: 'El lector de huellas está listo.',
  desconectado: 'No se detecta el lector de huellas. Verifica su conexión USB.'
})

// Nunca interpolar errores arbitrarios en el DOM. Los originales permanecen
// en los logs de la frontera IPC y en los logs del supervisor.
export function mensajeErrorLector(error, alternativa = MENSAJES_LECTOR.captura) {
  const texto = String(error)
  if (/SGFPM_OpenDevice|SGFPM_Init|\.dll|DLLLOAD|No se encontró bin/i.test(texto)) return MENSAJES_LECTOR.inicio
  if (/SGFPM_MatchTemplate|SGFPM_SetTemplateFormat/i.test(texto)) return MENSAJES_LECTOR.verificacion
  if (Object.values(MENSAJES_LECTOR).includes(texto)) return texto
  return alternativa
}

export function detalleLectorPublico(lector) {
  if (Object.values(MENSAJES_LECTOR).includes(lector?.detalle)) return lector.detalle
  if (lector?.estado === 'disponible') return MENSAJES_LECTOR.disponible
  if (lector?.estado === 'no_disponible') return MENSAJES_LECTOR.inicio
  return lector?.sdk_inicializado ? MENSAJES_LECTOR.esperando : MENSAJES_LECTOR.comprobando
}

function mensajeNegocio(error, permitidos, alternativa) {
  const texto = String(error).replace(/^(?:Error:|error:)\s*/i, '')
  if (/[a-z]:[\\/]|\\\\|\/[^\s/]+\/|SGFPM|SQLCipher|DPAPI|Tauri|Rust|\.dll|sqlite|database/i.test(texto)) return alternativa
  return permitidos.some(inicio => texto.startsWith(inicio)) ? texto : alternativa
}

export function mensajeErrorAsistencia(error) {
  return mensajeNegocio(error, ['Membresía pendiente', 'Membresía vencida', 'Acceso denegado: cliente desactivado', 'Cliente desactivado', 'Cliente no encontrado', 'Ya registró asistencia hoy', 'Contraseña incorrecta', 'Configura primero la contraseña administrativa'], 'No se pudo registrar la asistencia. Intenta nuevamente o consulta al operador.')
}

export function mensajeErrorAlta(error) {
  return mensajeNegocio(error, ['La huella ya pertenece al cliente #', 'Nombre inválido', 'Teléfono inválido', 'Fotografía demasiado grande', 'Formato de fotografía inválido', 'Fecha de registro inválida', 'Fecha de membresía inválida', 'El fin de membresía', 'Selecciona un', 'La membresía seleccionada no está disponible'], mensajeErrorLector(error, 'No se pudo guardar el miembro. Revisa los datos e intenta nuevamente.'))
}

export function mensajeErrorRenovacion(error) {
  return mensajeNegocio(error, ['Selecciona una membresía vigente', 'Cliente no encontrado o desactivado', 'Fecha de inicio almacenada inválida', 'Fecha de vencimiento almacenada inválida'], 'No se pudo renovar la membresía. Revisa el plan e intenta nuevamente.')
}
