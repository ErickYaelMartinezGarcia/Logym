# Diagnóstico del lector al iniciar con USB conectado

Fecha de inspección: 8 de noviembre de 2026. Rama: `feat/1.1.0-registro-sin-huella`. No se genera commit ni se publica esta revisión.

La auditoría adicional de propiedad, comparación con 1.0.0 y reinicios del CLI está en [ARRANQUE_Y_PROPIEDAD_LECTOR.md](ARRANQUE_Y_PROPIEDAD_LECTOR.md). Incluye el modo optativo de trazas por PID, hilo, manager, controlador y motivo. Es diagnóstico, sin un nuevo parche de recuperación.

## Hechos comprobados

- El paquete local **FDx SDK Pro for Windows v4.3.1**, `Inc/sgfplib.h:132`, define `SGFDX_ERROR_DEV_ALREADY_OPEN = 59` con el comentario **Device Exclusive access Error**. No significa ausencia de controladores. Los errores de carga del driver tienen otras constantes, como 6 y 56.
- El SHA-256 de `src-tauri/bin/sgfplib.dll` coincide con `bin/x64/sgfplib.dll` de ese paquete: `8F06D5A827FACD202E8140ED9FBE892BA6816A8507F17055A7C83BE21F403CFA`. La versión del archivo es 4.8.4.2. La definición se contrastó con el SDK que corresponde a la DLL utilizada, no solamente con otra API.
- Como corroboración pública, el [manual oficial SecuGen WebAPI](https://webapi.secugen.com/docs/secugen_web_service_api_doc.pdf), tabla de errores, describe 59 como dispositivo ocupado. WebAPI es otra interfaz: para el contrato nativo se tomó como fuente principal el header local anterior.
- Durante la inspección había un proceso `sgibiosrv.exe`, PID 9540, iniciado antes de Logym, y estaba activo el servicio de biometría de Windows. No había un proceso `logym.exe` en ese momento. Esto es un snapshot, no una prueba retrospectiva del dueño del dispositivo durante el error reportado.
- La secuencia nativa actual es Create → EnumerateDevice → Init(modelo enumerado) → SetTemplateFormat → OpenDevice(USB_AUTO_DETECT) → GetDeviceInfo. Se contrastó con el header y las referencias oficiales incluidas en el SDK.
- `GLOBAL_MANAGER` serializa inicialización, apertura, captura, comparación y destrucción dentro del proceso. `INICIADO` limita el supervisor a una instancia; `RECONEXION` coordina pausa e inicio. La aplicación también mantiene el mecanismo existente de instancia única. No se encontró un camino que abra simultáneamente dos managers dentro de Logym.
- Si OpenDevice falla, el propietario RAII termina el manager. No llama CloseDevice sobre una apertura rechazada, porque no adquirió ese dispositivo. Si la apertura tuvo éxito, ejecuta CloseDevice antes de Terminate al liberar. Cada nuevo intento parte de un manager nuevo tras un fallo de inicialización.

## Causa probable y límites

La evidencia apunta a **contención por acceso exclusivo**, posiblemente con `sgibiosrv.exe` u otra aplicación biométrica. La presencia de ese proceso, el significado exacto de 59 y que reconectar USB permita recuperar son compatibles con esa hipótesis. El código no identifica por sí solo al proceso propietario; tampoco permite distinguir definitivamente entre otro proceso y un estado residual del controlador.

No se afirma que `sgibiosrv.exe` sea el responsable demostrado. En la inspección inicial no se detuvieron procesos o servicios, no se reinició USB y no se abrió el lector. La suite usa funciones nativas simuladas, de modo que no reproduce por sí sola la situación física. Posteriormente se realizó la prueba aislada descrita abajo, con las DLL reales y sin captura de huellas ni acceso a bases de datos.

Se conserva el orden de llamadas, la exclusión mutua, la liberación RAII, los reintentos y los tiempos de recuperación. No se implementan cierres forzados de sesiones ajenas ni cambios de driver. Los cambios en el wrapper y el supervisor se limitan a diagnóstico y presentación de mensajes; no cambian el binario del SDK ni la lógica de recuperación.

## Diagnóstico interno y mensajes públicos

Los logs internos del lector conservan el código y la función exactos. Ahora incluyen fecha, PID, contexto, estado/revisión del supervisor, presencia USB, evidencia de inicialización/captura y generación USB. En la apertura se añade el modelo enumerado, ID y arquitectura; en un fallo de inicialización se conserva la ruta de la DLL. Las liberaciones registran CloseDevice/Terminate y sus resultados, para rastrear el ciclo de vida de cada manager.

En desarrollo estos mensajes se escriben en stderr, como los logs técnicos existentes. No se crea un archivo de registro persistente nuevo. Los logs originales de errores de captura y comparación también permanecen disponibles en la consola del frontend.

El módulo Rust `lector_mensajes.rs` y el traductor de presentación `src/services/mensajesLector.js` evitan mostrar diagnósticos arbitrarios en el DOM. Se cubren registro, check-in, indicador global, tooltip y notificaciones. Para una apertura fallida se presenta exactamente:

> No se pudo iniciar el lector de huellas. Verifica su conexión USB.

Los textos públicos no incluyen rutas, códigos, nombres de funciones ni referencias a SQLCipher, DPAPI, Rust o Tauri. Tampoco atribuyen el problema a drivers sin una identificación específica. Los rechazos legítimos de duplicidad, vigencia y autorización siguen siendo visibles.

## Validación y reproducción física pendiente

Las pruebas automatizadas comprueban que un OpenDevice 59 termina el manager sin cerrar un dispositivo ajeno, que varios fallos no acumulan handles y que un nuevo intento puede abrir y capturar al desaparecer el bloqueo simulado. Continúan pasando las pruebas de concurrencia, desconexión/reconexión, comparación y duplicidad existentes. También se prueban los logs y la ausencia de diagnósticos técnicos en los mensajes y el tooltip renderizado.

Para confirmar o descartar la hipótesis en la PC afectada:

1. Conservar una ejecución fallida con el lector conectado desde antes del inicio, anotando la hora y guardando los logs `[LECTOR]`. Registrar qué programas biométricos estaban activos.
2. Cerrar Logym normalmente para ejecutar su liberación. En un entorno de prueba, cerrar voluntariamente las otras aplicaciones biométricas, incluido el servidor WebAPI si se utiliza, desde sus controles habituales. Esta investigación no lo hace automáticamente.
3. Mantener el USB conectado y volver a abrir Logym. Comparar el resultado con la primera ejecución. Si abre sin reconectar, esto refuerza la hipótesis de contención.
4. Si persiste 59, conservar los logs de apertura y liberación y comparar con una prueba tras reconexión física. Revisar el propietario del dispositivo con herramientas de diagnóstico del sistema o soporte de SecuGen, antes de cambiar controladores o forzar cierres.
5. Probar captura, dos muestras coincidentes, duplicidad biométrica y asistencia después de recuperarse. Verificar que los mensajes al operador siguen siendo claros y no muestran los detalles internos.

No se declara corregida la causa física ni lista la versión para producción hasta completar esta comprobación con hardware real.

## Log real y prueba aislada posterior

El log aportado por el usuario entre las 13:18 y 13:19 muestra un mismo fallo OpenDevice 59 repetido aproximadamente cada cinco segundos. `SGFPM_Terminate: 0` informa liberación correcta del manager rechazado. Los números `revision`, `generacion_usb`, PID y direcciones del manager son metadatos, no errores adicionales.

Al retirar USB cambia la generación a 2 y se observa ausencia del lector. Al reconectarlo cambia a 3 y se obtiene `SGFPM_OpenDevice: 0`. Los mensajes `Can't open the key`, `LiveFingerLogEnable` y `LiveFingerLevel` aparecen antes de esa apertura correcta: en esa ejecución no bloquearon OpenDevice. Son mensajes internos de la librería sobre lecturas de configuración; no se interpreta su código 6 como si fuera el código de retorno de SGFPM_Init. No se cambian claves del registro ni parámetros de detección de dedo vivo.

A las 13:22 se ejecutó `.artifacts/diagnostico/Probe-Lector.ps1` con las DLL actuales. Logym estaba cerrado; el programa de prueba únicamente creó, enumeró, inicializó, intentó abrir y liberó su propio manager. No capturó imágenes ni plantillas.

| Condición | OpenDevice automático (957) | OpenDevice con ID enumerado (0) |
| --- | --- | --- |
| WebAPI activo, mismo USB conectado | 59 | 59 |

En ambas pruebas Create, EnumerateDevice, Init y SetTemplateFormat devolvieron 0; Terminate devolvió 0 tras el rechazo de apertura. La reproducción fuera de Logym descarta que esta incidencia concreta dependa de su interfaz Vue o de su supervisor. No descarta un problema externo de exclusión o del controlador.

Windows reportó `SecuGen U20 USB FRD`, estado `OK`, `ConfigManagerErrorCode=0`. Se confirmó que `SgiBioSrv` tiene una entrada en `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` y arranca con Windows. No se cambió esa entrada.

El resultado nativo completo está en `.artifacts/diagnostico/con-webapi.json`, fuera de Git. La comparación con WebAPI cerrado queda pendiente de autorización del usuario; no se debe forzar el cierre de otro programa sin ella.
