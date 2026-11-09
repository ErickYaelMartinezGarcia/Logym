# Renovaciones, Finanzas y Dashboard — revisión 1.1.0

Rama: `feat/1.1.0-registro-sin-huella`. Cambios posteriores a `0e1ffbf70954063511d86106cc4647ed641c89fe`, sin commit, push, merge ni instalador nuevo.

## Implementación

`RenovacionMembresia.vue` y `useRenovacion.js` centralizan la selección de plan, precio, confirmación y reintento. Se abren desde la búsqueda/tabla del Dashboard, membresías críticas, la lista/buscador de Administración de clientes, Editar cliente y Check-in manual o biométrico. El catálogo excluye planes desactivados y exclusivos de alta. Se consulta el cliente actual antes de confirmar y Rust verifica nuevamente su estado.

Se conserva el servicio y cálculo de renovación existentes: una anticipada extiende la vigencia pagada; una vencida comienza hoy. El pago y el historial se insertan en la misma transacción, sin inscripción. No se crean clientes nuevos al renovar. Se reutiliza `logym://datos-actualizados` para actualizar listas, métricas, gráficas y Finanzas; no se agrega polling.

Cada confirmación guarda un ID aleatorio y sus identificadores de cliente/plan antes del envío. Rust serializa la transacción y conserva el resultado de esa operación; un reintento, incluso concurrente o tras reiniciar, no extiende nuevamente ni cobra otra vez. Una respuesta técnica incierta conserva la confirmación pendiente y bloquea otra operación hasta recuperarla. El almacenamiento del navegador no conserva nombres, teléfonos, huellas ni contraseñas. Si no puede guardar la confirmación, no envía el cobro. Los rechazos explícitos de cliente desactivado o plan no disponible, anteriores a cualquier escritura, permiten seleccionar nuevamente.

Finanzas incorpora Todos, Nuevas membresías, Renovaciones, Visitas ocasionales y Semanales, conservando los rangos de fechas. Muestra el total general del periodo, el de la categoría y su desglose, serie y movimientos individuales con fecha, cliente/ID, concepto, inscripción e importe. Los semanales se identifican exclusivamente por una duración contratada persistida de siete días. Es un subconjunto de altas/renovaciones, no otra suma en el total general. La inscripción del alta se incluye en ese mismo movimiento y se muestra por separado en el desglose.

En Dashboard se elimina Registro de la tabla de búsqueda, se compactan columnas y se añaden acciones de renovación para vencidas/próximas. Las vencidas se filtran por 1, 2 o 3 meses calendario o Todas, con 3 por defecto; el límite es inclusivo y se ajusta al último día del mes cuando es necesario. Esto no modifica clientes ni afecta las próximas a vencer, cuyo umbral sigue siendo de cero a siete días. El contador de vencidas corresponde al filtro visible.

Los badges de estado del Dashboard tienen 100 px de ancho, altura mínima de 30 px y padding lateral de 12 px. En Check-in, las etiquetas completas de membresía disponen de un ancho mínimo de 170 px cuando cabe y padding lateral de 14 px; pueden crecer en altura para no desbordarse. Los de plan mantienen 106 × 32 px, sin fondo y con borde azul. Los nombres largos conservan su texto accesible y muestran el nombre completo mediante tooltip. Renovar usa un botón con icono, relieve y foco visible, separado visualmente de los badges. Los nombres de clientes y sus encabezados se alinean a la izquierda en todas las tablas; los demás datos permanecen centrados. Las tablas contienen su desplazamiento horizontal en pantallas pequeñas.

El Check-in principal ajusta su altura al espacio bajo la barra superior, con el panel derecho ligeramente más ancho (hasta 470 px) y sin scroll de página en las resoluciones comprobadas. El indicador es más grande, incorpora un tercer círculo interior y un rayo SVG centrado. Una sola animación de respiración mueve el conjunto, sin animaciones independientes desfasadas; respeta la preferencia de movimiento reducido. Los resultados de búsqueda y los diálogos mantienen su desplazamiento interno cuando hace falta para acceder a todos sus controles.

Mensual encabeza los catálogos de Registro, Renovación y Administración de precios y es la selección inicial si está disponible. Las confirmaciones pendientes conservan el plan originalmente seleccionado para evitar cambiar un cobro al reintentarlo. En Finanzas, Categoría de ingresos comparte el encabezado con el selector de periodo.

Periodos destacados muestra los siete días locales consecutivos, incluidos ceros, en orden cronológico cuando está seleccionada Semana. El backend también devuelve esa serie completa y suma asistencias de miembros y visitas una sola vez. Los modos de horas y meses conservan su comportamiento.

Registro cambia la modalidad por un checkbox «Registrar sin huella», inicialmente desmarcado. Se conservan las dos capturas, la comparación y el descarte de muestras al cambiar de modalidad. Ya no muestra ni envía Fecha de registro: Rust asigna la fecha/hora local e ignora cualquier fecha de alta suministrada por IPC.

## Persistencia y compatibilidad

Es necesaria una migración aditiva a `user_version=3` para conservar idempotencia y clasificación histórica verificable:

- `pagos.duracion_meses` y `pagos.duracion_dias` guardan la duración contratada en los nuevos cobros. Los pagos anteriores permanecen con `NULL`; no se infieren duraciones a partir del nombre ni del catálogo actual.
- `renovaciones_operaciones` conserva ID, cliente, plan y fin de la renovación confirmada. Se inserta en la misma transacción que el pago y el historial.

No se reconstruyen tablas históricas ni se recalculan importes anteriores. Se conservan la migración anterior, las tablas/vistas de visitas y la copia cifrada previa a migrar. El mecanismo de respaldo ya utiliza la constante de versión compartida y exporta todas las tablas: no se modifica su implementación, SQLCipher, DPAPI ni licencias. Las pruebas de respaldo/restauración incluyen la nueva tabla y las duraciones persistidas.

Un pago antiguo conserva su clasificación alta/renovación cuando la tiene persistida, pero no se incluye en Semanales si carece de duración histórica. La versión anterior rechaza una base v3: no hacer downgrade sobre la base migrada; se necesita el respaldo previo y considerar los movimientos posteriores.

## Pruebas realmente ejecutadas

| Comprobación | Resultado |
| --- | --- |
| Rust, `cargo test --locked --offline`, con DPAPI del perfil Windows | 118 aprobadas, 0 fallidas, 0 ignoradas |
| JavaScript, frontend y activación | 77 aprobadas, 0 fallidas |
| `cargo check --locked --offline` | PASS |
| Vue/Vite, frontend final | PASS |
| `scripts/verify-test-isolation.mjs`, assets del build actual | PASS, 26 archivos |
| `git diff --check HEAD` | PASS |
| Windows/Tauri debug, interfaz real + IPC real, base cifrada sintética | PASS |

Las nuevas pruebas cubren concurrencia/reapertura de una misma renovación, rollback del cobro y reintento, vigencias anticipadas/vencidas y semanales, clasificación financiera sin duplicar dinero, precios y duraciones históricos, migración v2→v3, fecha de alta del backend, filtro calendario, siete días con ceros, eventos del Dashboard y recuperación de confirmaciones tras reiniciar el frontend. Continúan pasando las pruebas de migración v1, respaldo, licencias y biometría simulada existentes.

La primera ejecución Rust en el aislamiento falló en ocho pruebas DPAPI por falta de acceso al perfil. Se repitió con ese acceso, sin cambiar cifrado ni validaciones. Se actualizaron las pruebas del antiguo formulario de renovación para ejercitar el módulo común real; las fronteras IPC/cámara de las pruebas JavaScript unitarias siguen simuladas.

La integración nativa reutilizó exclusivamente el perfil sintético `com.virtusgym.validation.3ae11414b1a01bec`, creado para la validación anterior, con DLL ausente del directorio aislado. Migró de v2 a v3. Desde la interfaz se confirmó una renovación anticipada semanal (15→22 de noviembre), otra vencida (9→15), y una desde Check-in manual (15→22), incluyendo doble clic sin cobro repetido y actualización visible sin recarga. También se abrieron los mismos controles desde lista y Editar cliente. Para el escenario vencido se ajustaron las fechas únicamente del cliente sintético mediante su editor normal.

En esa fixture, con $670 previos, los filtros reales mostraron: Todos $1,030/6 movimientos; Nuevas membresías $620/2; Renovaciones $360/3; Visitas $50/1; Semanales $360/3. La antigua alta semanal no se clasificó como semanal porque el esquema anterior no guardaba duración. Se comprobó el diseño a 780 px sin desbordamiento de página. Las capturas, snapshots y logs están en `.artifacts/renovaciones/`, fuera de Git.

La segunda apertura nativa conservó $1,030 sin añadir cobros. Se verificaron el checkbox inicialmente desmarcado, las dos fechas visibles de membresía y el cierre por Escape del modal. La medición del CSS renderizado comprobó las tres variantes de estado de 100 × 30 px (usando nodos de prueba del mismo badge), los dos planes de 132 × 32 px y el botón de 108 × 36 px. La captura final muestra los estados reales activa/vencida de la fixture; para esa revisión visual solo se cambió la fecha del cliente sintético anterior mediante Editar cliente.

Los ajustes visuales posteriores reducen los planes a 106 px, conservan el fondo ausente indicado por el usuario y alinean los nombres a la izquierda. Se verificaron por compilación Vue/Vite y revisión del CSS generado; no se repitió la integración nativa por estos ajustes de presentación. Dos pruebas JavaScript nuevas verifican el orden Mensual y su selección inicial, incluido el respeto al plan de una operación pendiente.

El ajuste posterior de Check-in se comprobó en Edge con Vue/CSS reales e IPC simulado, sin iniciar Rust ni abrir bases: 1366×768, 1280×720, 900×620, 780×705 y 520×620. No hubo scroll de página ni controles recortados. Se midió el mismo centro del rayo SVG y del círculo interior en tres fases de la única animación de respiración (0, 1500 y 3000 ms). Una etiqueta larga de estado mantuvo padding de 14 px a cada lado y su texto dentro del badge. Evidencia: `.artifacts/checkin-pantalla/layout-results.json` y capturas de esa carpeta. No es una prueba física del lector.

## Archivos modificados

- Renovación compartida: nuevos `src/components/RenovacionMembresia.vue` y `src/composables/useRenovacion.js`; conexión en `src/App.vue`, `src/views/{Dashboard,AdministracionClientes,CheckIn}.vue` y `src/composables/useCheckIn.js`.
- Dashboard y presentación: nuevo `src/services/periodosDashboard.js`, `src/composables/useDashboard.js` y `src/style.css`.
- Finanzas y registro: `src/views/{AdministracionPrecios,Registro}.vue`, `src/composables/useRegistroCliente.js` y `src/services/tauriApi.js`.
- Backend: `src-tauri/src/{commands,database,membresias,models,repositories,services}.rs`.
- Pruebas: `tests/{frontend,activation}.test.mjs` y `src-tauri/src/tests/{database,membresias,repositories,services}.rs`.
- Informe: este documento. Los scripts, bases y capturas de integración permanecen fuera de Git.

## Límites

No se ha ejecutado biometría física en esta revisión. Las pruebas biométricas usan SDK simulado; no se modifica el SDK, supervisor, exclusión mutua ni matcher 1:1. La integración nativa usa debug y no valida activación de release ni instalación en una computadora limpia. No se usa la base del gimnasio y no se genera instalador. El instalador de la revisión anterior no contiene estos cambios.

Antes de publicar, revisar esta interfaz y aceptar la migración v3 en una copia de prueba. Los reintentos recuperan la misma operación; una renovación deliberada posterior utiliza otro ID y produce un nuevo cobro.
