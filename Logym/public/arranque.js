// Disponible antes de descargar Vue y sus rutas. No depende del backend.
(() => {
  let terminado = false
  const estado = document.getElementById('startup-status')
  const reintentar = document.getElementById('startup-retry')
  const lento = setTimeout(() => {
    if (terminado) return
    estado.textContent = 'La apertura está tardando más de lo habitual. Puedes esperar o volver a intentar.'
    reintentar.hidden = false
  }, 15000)
  function fallo(evento) {
    if (terminado) return
    console.error('[ARRANQUE] No se pudo cargar la interfaz:', evento.error || evento.reason || evento.detail || evento)
    clearTimeout(lento)
    estado.textContent = 'No se pudo abrir la aplicación. Vuelve a intentar.'
    reintentar.hidden = false
    document.querySelector('.app-startup')?.setAttribute('aria-busy', 'false')
  }
  function listo() {
    terminado = true
    clearTimeout(lento)
    window.removeEventListener('error', fallo, true)
    window.removeEventListener('unhandledrejection', fallo)
    window.removeEventListener('logym:arranque-error', fallo)
  }
  reintentar.addEventListener('click', () => window.location.reload())
  window.addEventListener('error', fallo, true)
  window.addEventListener('unhandledrejection', fallo)
  window.addEventListener('logym:arranque-error', fallo)
  window.addEventListener('logym:interfaz-lista', listo, { once: true })
})()
