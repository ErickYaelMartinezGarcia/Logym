import { onBeforeUnmount, ref } from 'vue'

export function useCamera(mostrarMensaje) {
  const videoRef = ref(null)
  const canvasRef = ref(null)
  const fotoBase64 = ref('')
  const stream = ref(null)
  const camaraActiva = ref(false)
  const camaraDisponible = ref(true)

  async function iniciarCamara() {
    if (!navigator.mediaDevices?.getUserMedia) {
      camaraDisponible.value = false
      mostrarMensaje('Este entorno no expone la cámara. Usa localhost o la aplicación Tauri.', 'error')
      return
    }
    try {
      stream.value = await navigator.mediaDevices.getUserMedia({ video: { facingMode: 'user', width: { ideal: 1280 } }, audio: false })
      videoRef.value.srcObject = stream.value
      camaraActiva.value = true
      mostrarMensaje('Cámara lista. Mira al frente y captura tu foto.', 'success')
    } catch (error) {
      const razon = error.name === 'NotAllowedError' ? 'Permiso de cámara rechazado.' : error.name === 'NotFoundError' ? 'No se encontró una cámara conectada.' : 'No fue posible abrir la cámara.'
      mostrarMensaje(`${razon} Revisa permisos del navegador o del sistema.`, 'error')
    }
  }

  function detenerCamara() {
    stream.value?.getTracks().forEach((track) => track.stop())
    stream.value = null
    camaraActiva.value = false
  }

  function capturarFoto() {
    const video = videoRef.value
    const canvas = canvasRef.value
    if (!camaraActiva.value || !video?.videoWidth || !canvas) {
      mostrarMensaje('Primero activa la cámara y espera a que aparezca la imagen.', 'error')
      return
    }
    canvas.width = video.videoWidth
    canvas.height = video.videoHeight
    canvas.getContext('2d').drawImage(video, 0, 0, canvas.width, canvas.height)
    fotoBase64.value = canvas.toDataURL('image/jpeg', 0.86)
    mostrarMensaje('Foto capturada correctamente.', 'success')
  }

  onBeforeUnmount(detenerCamara)

  return { videoRef, canvasRef, fotoBase64, camaraActiva, camaraDisponible, iniciarCamara, detenerCamara, capturarFoto }
}
