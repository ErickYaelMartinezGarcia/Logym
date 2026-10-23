# VIRTUS GYM

Panel local para controlar miembros, acceso biométrico y registro de asistencia de VIRTUS GYM. La interfaz está construida con Vue 3 + Vite y la persistencia local se realiza con SQLite desde Tauri.

## Desarrollo

```bash
npm install
npm run dev
```

Para probar cámara en el navegador abre el origen que imprime Vite, normalmente `http://localhost:5173`. `getUserMedia()` solo funciona en un contexto seguro: `localhost` cuenta como origen seguro, pero abrir `dist/index.html` directamente con `file://` no. También hay que aceptar el permiso de cámara en el navegador y verificar que ninguna otra aplicación la esté usando.

## Cámara web

La pantalla **Nuevo miembro** no solicita la cámara automáticamente. Esto es intencional: el navegador y Tauri pueden bloquear solicitudes silenciosas o mostrarlas en un momento inesperado. El flujo actual es:

1. Pulsar **Activar cámara**.
2. `navigator.mediaDevices.getUserMedia({ video, audio: false })` solicita permiso y conecta el `MediaStream` al elemento `<video>`.
3. Pulsar **Tomar foto** para dibujar el frame actual en un `<canvas>` y convertirlo a JPEG base64.
4. Al salir de la pantalla se detienen todas las pistas (`track.stop()`), evitando que la cámara quede encendida.

Si no se abre en una laptop con cámara, el mensaje diferencia entre: API no disponible (origen inseguro o WebView sin soporte), permiso rechazado, dispositivo inexistente y otros errores. En Chrome/Edge se puede revisar el candado de la URL y **Configuración del sitio > Cámara**. En la aplicación Tauri debe probarse con `npm run tauri dev`, no abriendo un HTML compilado manualmente.

Para producción conviene reemplazar el base64 por un archivo administrado por la aplicación (por ejemplo, guardarlo en `appData`) y persistir únicamente la ruta. El base64 actual se mantiene como una implementación local sencilla.

## Lector de huellas

`src-tauri/src/main.rs` contiene hoy un punto de integración controlado:

- `capturar_huella` devuelve `template_fake_dp4500_*`; no lee hardware real.
- `guardar_template` mantiene temporalmente el template en memoria.
- `comparar_huella` compara texto y sirve únicamente para pruebas.

Para conectar un lector real (por ejemplo DigitalPersona 4500):

1. Instalar el SDK oficial y su driver en el equipo donde correrá Tauri.
2. Crear un módulo Rust que use el SDK o un pequeño servicio local del fabricante; evitar cargar DLLs directamente desde Vue.
3. Sustituir el cuerpo de `capturar_huella` por la secuencia del SDK: inicializar lector, esperar evento de dedo, adquirir imagen, crear template y devolver un identificador/template serializable.
4. Guardar el template cifrado o protegido en el almacenamiento local; no dejarlo en `PLANTILLAS`, porque ese mapa se pierde al cerrar la aplicación.
5. En el registro, capturar dos muestras y validar la calidad/coincidencia con el SDK antes de guardar.
6. En check-in, capturar una muestra y ejecutar la búsqueda 1:N contra los templates almacenados; registrar la asistencia solo si el SDK devuelve una coincidencia válida.
7. Implementar errores de lector desconectado, timeout, dedo retirado, mala calidad y cancelación para que la UI pueda mostrar estados específicos.

No es posible activar un lector de huellas desde `npm run dev`: el navegador no expone acceso genérico a lectores biométricos USB. El comando Tauri es el puente correcto para hardware, pero requiere el SDK y el driver del modelo elegido.

## Estructura visual

- `src/components/AppShell.vue`: menú hamburguesa y sidebar desplegable compartido.
- `src/views/Dashboard.vue`: resumen de métricas y miembros.
- `src/views/Registro.vue`: alta, cámara y captura biométrica.
- `src/views/CheckIn.vue`: acceso de miembros.
- `src/style.css`: identidad VIRTUS GYM, fondo oscuro y acentos rojo neón.
