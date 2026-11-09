# LOGYM 1.1.0 — entrega para revisión

Fecha: 8 de noviembre de 2026. Rama: `feat/1.1.0-registro-sin-huella`.

## Implementación

El registro conserva por defecto la modalidad con huella, las dos capturas, la comparación y el control de duplicidad que incluye clientes inactivos. La opción explícita «Registrar sin huella» oculta la captura y evita las llamadas de captura, comparación e inicialización del SDK en el alta. Cambiar de modalidad descarta capturas anteriores. Después de guardar, la modalidad vuelve al valor por defecto.

El contrato IPC agrega `sinHuella`, con valor por defecto `false` para conservar compatibilidad con llamadas existentes. En la modalidad sin huella la cadena de transporte vacía representa ausencia de captura y se persiste exclusivamente como SQL `NULL`; nunca se almacena como una plantilla válida. Rust rechaza una plantilla presente cuando se solicita alta sin huella, y sigue exigiendo una plantilla hexadecimal de 800 caracteres en la modalidad biométrica.

El alta sigue usando la transacción existente para cliente, historial de membresía y pago. No se cambian las validaciones de datos, fechas, planes, inscripción ni precios. Una falla al insertar el pago revierte todo el alta.

Check-in incorpora una barra de búsqueda por nombre, teléfono o ID encima del lector, en la columna derecha, con máximo de 30 resultados entre clientes activos. Al buscar, los resultados reemplazan visualmente al lector dentro del mismo espacio, sin modificar la columna izquierda. «Volver al lector» cancela la consulta y descarta respuestas pendientes. La búsqueda no devuelve huellas.

La selección abre la modal existente con foto ampliada a 300 × 300 px en escritorio o avatar neutro, nombre, ID, teléfono e inicio y fin de membresía. Por instrucción del usuario, la asistencia manual se confirma únicamente con «Registrar asistencia», sin contraseña. La modal omite la fecha de registro y el recuadro verde de éxito para dar prioridad a la identificación por fotografía. La modalidad manual no muestra ni ejecuta renovaciones. Tras el éxito se conserva la indicación de asistencia registrada durante dos segundos y se vuelve al lector; un rechazo conserva su motivo visible y permite regresar.

La confirmación llama al comando existente `registrar_asistencia`, con verificación de licencia y notificación de actualización. El backend vuelve a comprobar estado activo y vigencia y aplica la misma prevención de duplicados diarios usada por el supervisor biométrico. También admite seleccionar clientes con huella para compartir ese control entre métodos. No se cambian las reglas de captura ni recuperación del supervisor.

La búsqueda y selección manual no pausan el lector. Si llega una lectura mientras se muestran resultados, se presenta su modal biométrica conservando la búsqueda. Si llega durante la modal manual o mientras hay otra lectura visible, el resultado se conserva en una cola y se muestra al cerrar la modal actual. La pausa por un evento biométrico conserva el comportamiento existente; al terminar la cola se libera. La interfaz no vuelve a registrar esas lecturas, pues el backend ya resolvió sus asistencias.

La foto ya era opcional en el registro con y sin huella. Se conserva el código de cámara, formulario y validación de fotos; se verifican cámara ausente, permiso denegado y alta con fotografía vacía.

Los diagnósticos del lector se separan ahora de los mensajes públicos, incluidos indicador global, tooltip, notificaciones, captura, comparación y errores de asistencia. El análisis del error de apertura está en `docs/DIAGNOSTICO_LECTOR_59.md`.

## Esquema y protección de datos

No se necesita migración: `clientes.template_huella` ya admite `NULL`. `user_version` permanece en 1. No se cambian SQLCipher, DPAPI, licencias, identidad de hardware, esquemas ni registros históricos. Las pruebas usan únicamente datos sintéticos y directorios temporales; no se abre una base real de gimnasio.

## Archivos modificados

- Registro: `src/views/Registro.vue`, `src/composables/useRegistroCliente.js`.
- Asistencia: `src/views/CheckIn.vue`, `src/composables/useCheckIn.js`, `src/services/tauriApi.js`.
- Backend: `src-tauri/src/models.rs`, `services.rs`, `repositories.rs`, `commands.rs`, `main.rs`.
- Pruebas: `tests/frontend.test.mjs`; `src-tauri/src/tests/services.rs`, `repositories.rs`, `database.rs`, `secugen.rs`, `test_support.rs`.
- Versión 1.1.0: `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src/components/AppShell.vue`.
- Informe: `docs/REVISION_1.1.0.md`.
- Arranque visible y auditoría: `index.html`, `public/arranque.js`, `src/main.js`, `src/router.js` y `docs/ARRANQUE_Y_PROPIEDAD_LECTOR.md`, además de las trazas y presentación en los archivos ya enumerados. Se conserva el alcance de solo diagnóstico para el lector.
- Diagnóstico y mensajes: `src/App.vue`, `src/services/mensajesLector.js`, `src-tauri/src/lector_mensajes.rs`, `src-tauri/src/lector_estado.rs`, `src-tauri/src/lector_worker.rs`, `src-tauri/src/secugen.rs`, `src-tauri/src/tests/lector_estado.rs`, `tests/activation.test.mjs`, `docs/DIAGNOSTICO_LECTOR_59.md`.

## Resultados reales

Ejecución final: `node scripts/run-tests.mjs` en Windows con acceso al perfil del usuario.

| Comprobación | Resultado |
| --- | --- |
| `cargo check --locked` | PASS |
| `cargo test --locked` | 103 aprobadas, 0 fallidas, 0 ignoradas |
| JavaScript, incluidos activación y flujos existentes | 63 aprobadas, 0 fallidas |
| `npm run build -- --outDir .artifacts/validation-1.1.0` | PASS |
| Interfaz real en Edge con IPC simulado | PASS; escritorio y móvil de 390 px |

Los logs y el resumen están en `.artifacts/tests/latest/`; el build de validación está en `.artifacts/validation-1.1.0/`. Estas carpetas no se incluyen en Git. No se genera instalador ni lanzamiento definitivo.

Las capturas y resultados de navegador están en `.artifacts/ui-check/`. Se comprueban reemplazo de tarjeta sin cambio de altura, selección, foto/avatar ampliados, botón de asistencia sin contraseña, éxito sin recuadro y regreso automático, inicio y fin de membresía, rechazo de membresía vencida sin controles de renovación y ausencia de desbordamiento horizontal a 390 px. Son pruebas con clientes sintéticos y llamadas nativas simuladas.

La primera ejecución dentro del aislamiento tuvo siete fallas de DPAPI (`os error 2`), incluidas pruebas existentes. Se investigó y repitió sin el aislamiento; todas pasaron sin modificar las validaciones ni el cifrado.

Las pruebas cubren alta con/sin huella, duplicidad biométrica, asistencia vigente/vencida/pendiente/inactiva, duplicados entre métodos, reapertura de SQLite y SQLCipher/DPAPI con huella nula, preservación de una base existente y sus registros históricos, importes y rollback ante errores. El SDK se simula en las pruebas biométricas: no sustituyen la prueba física.

## Prueba manual requerida

1. Ejecutar `npm run tauri dev` en un entorno de prueba Windows con licencia válida y lector SecuGen configurado. Usar clientes de prueba y una base de prueba; el alta cobra según el plan e inscripción seleccionados.
2. Abrir Registro. Confirmar que «Registrar con huella» es la opción inicial. Completar datos y plan; capturar dos veces el mismo dedo y guardar. Verificar cliente, vigencia, historial y pago. Repetir ese dedo en otro alta y comprobar el rechazo sin cobro adicional.
3. Abrir Registro y seleccionar «Registrar sin huella». Confirmar que desaparecen las capturas. Completar datos y plan y guardar. Verificar que no exige lector y que aparecen cliente, membresía y pago. Reiniciar la aplicación y comprobar que el cliente persiste.
4. En Check-in, usar la barra sobre el lector para buscar al cliente sin huella por nombre, teléfono o ID. Comprobar que los resultados ocupan la tarjeta del lector. Seleccionarlo, verificar foto ampliada/ID/teléfono/inicio y fin de membresía y pulsar «Registrar asistencia», sin contraseña. La selección por sí sola no debe registrar asistencia. Tras confirmar debe indicarse el éxito y volver al lector después de dos segundos. Repetir y comprobar que no se duplica.
5. Con el cliente biométrico, registrar acceso colocando el dedo. Intentar después confirmar manualmente el mismo cliente y comprobar el rechazo por duplicidad. Con otro cliente o al día siguiente, probar el orden inverso.
6. Probar cliente vencido, pendiente y desactivado. Vencidos y pendientes pueden encontrarse, pero su asistencia debe rechazarse. Los inactivos no aparecen; si se desactiva al cliente después de seleccionarlo, el backend debe rechazar la confirmación.
7. Probar dos clientes con el mismo nombre: elegir por ID, teléfono y fotografía. Cambiar la consulta y comprobar que se elimina la selección anterior. Comprobar que pagos, renovaciones y reportes anteriores se conservan.
8. Mientras se muestran resultados, colocar una huella registrada y comprobar que aparece su resultado sin perder la consulta. Durante una confirmación manual, colocar otra huella y comprobar que su resultado queda pendiente sin cambiar el cliente seleccionado; al cerrar debe mostrarse, sin volver a guardar su asistencia.
9. Probar ambos registros sin foto, sin cámara o con permiso denegado. La modal manual debe mostrar el avatar cuando no existe fotografía. Probar el arranque con el USB ya conectado siguiendo `docs/DIAGNOSTICO_LECTOR_59.md`.

## Limitaciones y estado de revisión

Pendiente la validación física del registro con huella, sin huella y de ambos métodos de asistencia por el usuario. No se declara lista para producción.

La búsqueda usa `lower()` de SQLite: no normaliza acentos y su conversión de mayúsculas no ASCII es limitada. Si un nombre acentuado no coincide, buscar con su grafía almacenada, por teléfono o ID. Los resultados se limitan a 30; precisar la consulta cuando sea necesario. La identidad manual depende de la verificación del operador, especialmente cuando no existe fotografía.

Por instrucción del usuario se verificó que `c9ac56c7497982d0f4b72db2e58f25fd4fd3546a` era el último commit local/remoto y que el árbol estaba limpio. Se ejecutó `git reset --soft HEAD~1` y exclusivamente `git push --force-with-lease origin feat/1.1.0-registro-sin-huella` para retirarlo del remoto sin perder sus cambios.

HEAD local y origin quedan en `24ee5aaa7483844b6271f56e8cb4e7a63b37dc03`. Los cambios originales y sus correcciones se conservan preparados para revisión en staging. No se crea un nuevo commit ni se realiza otro push. `main` permanece intacta.
