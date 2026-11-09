import './style.css'

const inicio = performance.now()
async function iniciar() {
  try {
    const [{ createApp }, { default: App }, { default: router }] = await Promise.all([
      import('vue'), import('./App.vue'), import('./router')
    ])
    const app = createApp(App).use(router)
    await router.isReady()
    app.mount('#app')
    console.info(`[ARRANQUE] fase=interfaz_montada duracion_ms=${Math.round(performance.now() - inicio)}`)
    window.dispatchEvent(new Event('logym:interfaz-lista'))
  } catch (error) {
    console.error('[ARRANQUE] Fallo al cargar la interfaz:', error)
    window.dispatchEvent(new CustomEvent('logym:arranque-error', { detail: error }))
  }
}
iniciar()
