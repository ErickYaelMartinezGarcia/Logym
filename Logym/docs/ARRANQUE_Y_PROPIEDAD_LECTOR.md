# Arranque visible y auditoría de propiedad del lector

8 de noviembre de 2026. Rama `feat/1.1.0-registro-sin-huella`.

Esta revisión implementa la presentación del arranque. Para el lector únicamente añade instrumentación y diagnóstico: no modifica su secuencia de apertura, exclusión mutua, pausa, recuperación ni frecuencia de reintentos. No ejecuta cierres de otros programas o reinicios USB. No se crea commit, push o merge.

## Pantalla blanca y tiempo de apertura

El HTML anterior contenía un `#app` vacío, sin fondo ni contenido previo a Vue. `main.js` importaba el router, que importaba inmediatamente las cinco pantallas. Hasta cargar esos módulos y montar Vue no había contenido que mostrar. Esto está comprobado en el código; no equivale a medir todo el arranque de WebView2 en la PC afectada.

El comando de desarrollo también inicia Vite y compila Rust antes de ejecutar el binario. Una pantalla HTML no puede aparecer durante la compilación, cuando aún no existe el proceso de Logym. No se declara eliminada ni acelerada esa compilación.

Se implementa:

- HTML inicial oscuro con marca, indicador de carga y mensaje de apertura, independiente de Vue y del backend.
- Aviso si tarda más de 15 segundos y opción de reintentar. Un fallo conserva un mensaje visible sin mostrar el diagnóstico técnico.
- Ventana nativa con tema oscuro, inicialmente oculta; `on_page_load` la muestra cuando el documento ya está cargado. El HTML puede presentar la carga mientras terminan las importaciones asíncronas.
- Presentación equivalente mientras Vue verifica el acceso, conservando la verificación de licencia existente.
- Rutas diferidas: las otras pantallas se descargan al navegar.
- Logs `[ARRANQUE]` con PID y tiempo acumulado para inicio del proceso, setup, verificación, almacenamiento y documento cargado. La consola del frontend mide su montaje desde la ejecución de `main.js`.

El almacenamiento y su esquema mantienen el mismo orden en setup; no se trasladan a otro hilo. En desarrollo `licencia::verificar` retorna inmediatamente por `debug_assertions`, como ya ocurría antes: no se atribuye ese retraso a CIM sin evidencia.

La [guía oficial de Tauri v1](https://v1.tauri.app/v1/guides/features/splashscreen/) contempla presentar una pantalla de carga durante la inicialización. Aquí se mantiene una sola ventana.

## Inventario de llamadas y propiedad

Los punteros FFI se cargan una vez en `SgfpmApi`. Las llamadas reales están centralizadas en `src-tauri/src/secugen.rs`:

| Función | Lugar de llamada | Protección |
| --- | --- | --- |
| `SGFPM_Create` | `create_manager` | Desde `ensure_manager`, bajo `GLOBAL_MANAGER` |
| `SGFPM_Init` | `create_manager` | Modelo enumerado, mismo mutex |
| `SGFPM_OpenDevice` | `create_manager` | Después de Init y formato, mismo mutex |
| `SGFPM_CloseDevice` | `Drop for NativeManager` | Solo si OpenDevice tuvo éxito |
| `SGFPM_Terminate` | `Drop for NativeManager` | También ante errores de inicialización |

Las declaraciones de símbolos y los punteros simulados de las pruebas no constituyen aperturas adicionales.

`SecuGenController` es un wrapper que contiene la ruta de DLL y un ID diagnóstico. Pueden coexistir varios wrappers; ninguno posee ni destruye un handle nativo. El dueño es `NativeManager`, que no se clona, dentro de `GLOBAL_MANAGER: Mutex<Option<NativeManager>>`.

Las solicitudes de controlador proceden de `services::guardar_cliente` para un alta biométrica, `services::capturar_huella`, `services::comparar_huella` y `lector_worker::inicializar_lector`, usado por `buscar_cliente_por_huella_worker`. El registro sin huella y la asistencia manual no abren el lector.

`from_bin_path` mantiene el mutex durante inicialización y apertura. Captura, comparación e identificación mantienen el mismo mutex durante sus llamadas FFI. La destrucción/recreación también se serializa. No se encontró una apertura simultánea de dos managers en el proceso; las pruebas simuladas conservan un máximo de uno.

Esta garantía es interna al proceso. WebAPI y otros programas no comparten ese mutex. La instancia única de Logym utiliza `Local\VirtusGym.Logym.Production`: no arbitra todos los programas biométricos ni todas las sesiones de Windows.

## Supervisor, navegación y reinicios

- `INICIADO: OnceLock` limita el supervisor a uno por proceso.
- `RECONEXION` coordina pausa e inicio. Una pausa no garantiza que una captura anterior haya terminado; el acceso FFI sigue serializado por `GLOBAL_MANAGER`. El worker comprueba la pausa antes de registrar un resultado.
- Registro solicita pausa al montar y reanudación al salir. Son solicitudes asíncronas sobre un booleano global. Una navegación rápida podría intercalarlas; es un punto a observar, pero no demuestra dos aperturas ni explica por sí solo un 59. Se deja sin modificar por el alcance de diagnóstico.
- Check-in escucha eventos del supervisor. No crea un segundo lector por pantalla ni por selección manual.
- El cierre normal pasa por `ExitRequested`, solicita parada, espera los hilos y libera el manager. Puede esperar el timeout de una captura en curso.

El paquete instalado es `@tauri-apps/cli` 1.6.3. Su [fuente oficial](https://github.com/tauri-apps/tauri/blob/tauri-cli-v1.6.3/tooling/cli/src/interface/rust.rs) usa `child.kill()` en `DevChild::kill`. El watcher termina el proceso anterior, espera su salida y vuelve a ejecutarlo al recompilar; no solicita cerrar la ventana de Logym.

Se infiere que ese reinicio forzado puede omitir `ExitRequested` y los destructores Rust. Esto no prueba que Windows conserve handles del proceso indefinidamente ni que sea la causa del 59. El efecto sobre el controlador requiere prueba física. No se modifica el watcher ni el cierre como supuesto fix.

## Comparación con 1.0.0 y recuperación temporal

Se comparó con el commit estable `24ee5aaa7483844b6271f56e8cb4e7a63b37dc03`, sin cambiar esa referencia ni `main`.

La secuencia Create → Enumerate → Init → formato → Open → información, manager global, mutex, propiedad RAII, pausa, cierre y tiempos de recuperación ya existían en 1.0.0. Los cambios posteriores en esas rutas son diagnósticos, mensajes e IDs de correlación. La función sin huella no agrega otro propietario nativo.

La revisión anterior no corrigió una causa física demostrada: identificó el significado de 59, separó mensajes públicos, observó procesos y reprodujo el rechazo fuera de Logym. El log aportado muestra éxito tras reconectar USB; luego el usuario informó que funcionaba sin una prueba que aislara la causa. Esto no garantiza que desaparezca una contención externa o un efecto de reinicio y no justificaba prometer ausencia de recurrencia.

## Trazas para validar

Cada log conserva fecha, PID, hilo, motivo, estado y error original. `instancia_nativa` identifica un manager y `controlador_id` identifica un wrapper; las direcciones de memoria pueden reutilizarse. Los IDs se reinician en cada proceso: siempre correlacionarlos con PID.

El modo optativo `LOGYM_DIAGNOSTICO_LECTOR=1` añade inicio/fin de operación con duración, origen de wrappers, espera/adquisición del mutex, resultados de Init/Open/captura y cierre normal. Los motivos distinguen `checkin_global`, `registro_capturar`, `registro_comparar` y `registro_guardar`. No incluye imágenes, plantillas, nombres de clientes ni licencias.

Desde PowerShell, en la carpeta `Logym`:

```powershell
$env:LOGYM_DIAGNOSTICO_LECTOR = '1'
New-Item -ItemType Directory -Force '.artifacts/diagnostico' | Out-Null
npm run tauri dev *>&1 | Tee-Object -FilePath '.artifacts/diagnostico/sesion-lector.log'
```

`controlador_liberado` elimina solo un wrapper. CloseDevice/Terminate indican liberación del propietario. Al recompilar puede faltar `cierre_completado`; anotarlo junto con el cambio de PID.

## Pruebas y límites

Las pruebas simuladas cubren fallos de apertura y liberación, controladores concurrentes con un único manager, captura, identificación, duplicidad, pausa, cierre y reconexión. La nueva prueba verifica restauración de motivos anidados y mensajes públicos seguros. El arranque se prueba con módulos demorados/fallidos y backend simulado.

Resultados de esta revisión: `cargo check --locked` aprobado; 103 pruebas Rust y 63 JavaScript aprobadas; compilación frontend aprobada. El navegador comprobó HTML visible antes de Vue, fondo oscuro, carga demorada, fallo con reintento, montaje normal, rutas diferidas y navegación Registro ↔ Check-in. Resultados y capturas: `.artifacts/arranque/`. La traza de la prueba concurrente, exclusivamente con SDK simulado, está en `.artifacts/diagnostico/traza-simulada-concurrencia.log`; no es un log físico del lector.

No se ejecutó esta matriz con hardware real en esta revisión. Para cada caso conservar hora, PID, programas biométricos activos y log completo:

| Escenario físico pendiente | Qué observar |
| --- | --- |
| USB conectado antes del arranque | Resultado de Open, otros procesos y una instancia nativa abierta por PID |
| Arranque sin USB y conexión posterior | Generación y primera apertura; disponibilidad tras captura comprobada |
| Registro ↔ Check-in | Pausa/reanudación y reutilización del manager sin FFI simultáneas |
| Varias huellas consecutivas | Wrappers comparten manager; comparación y duplicidad siguen activas |
| Varios check-ins consecutivos | Capturas serializadas y asistencia única diaria |
| Desconexión/reconexión manual durante ejecución | Liberación del anterior y nueva instancia/generación; sin reinicio USB por software |
| Reinicio de Tauri en desarrollo | Rebuild, PID anterior/nuevo, marcas de cierre y primera apertura nueva |

Para el arranque comparar compilación de Cargo, `[ARRANQUE] tiempo_ms` del proceso y montaje del frontend. Confirmar en la ventana nativa que no hay destello blanco: el navegador no reproduce el primer pintado de WebView2. No se declara resuelta la recurrencia de 59 ni lista para producción la versión sin pruebas físicas.
