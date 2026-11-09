# Visitas ocasionales y planes en días — revisión 1.1.0

Rama: `feat/1.1.0-registro-sin-huella`. Base de esta revisión: `6abeb65003e100ea10fd6b5a2dc0acd9fc781142`.
Cambios sin commit, push ni merge. No se genera instalador en esta revisión.

## Uso

En Check-in, el botón flotante «Visita» abre un diálogo compacto. El nombre es opcional; se confirma una entrada por $50.00 MXN, sin cliente, membresía, fotografía ni biometría. El importe se fija en Rust y se restringe en SQLite a 5000 centavos. Una nueva entrada requiere otra operación y otro cobro. El recibo no es un pase ni permite registrar otra asistencia.

El diálogo nativo admite teclado, foco modal y Escape; impide cancelar durante el guardado. El botón no ocupa el buscador, y las notificaciones siguen en la esquina superior. Los errores públicos son genéricos y el diagnóstico se conserva en logs.

Para crear la semana: Administración de precios → Crear nuevo plan → nombre «Semanal», duración «Días», valor 7 y precio definido por el administrador → Crear plan. No se inventa un precio ni se inserta automáticamente un plan semanal. Tras guardarlo aparece en el catálogo de Registro y renovaciones. También se pueden editar y desactivar estos planes.

Los días son inclusivos: 8 de noviembre + 7 días termina el 14. Una renovación vigente comienza después del último día pagado y agrega siete días completos; una vencida comienza hoy y termina seis días después. Los meses mantienen su cálculo calendario anterior. El backend recalcula la fecha del alta desde el catálogo. La inscripción sigue siendo opcional en altas y cero en renovaciones.

## Persistencia y compatibilidad

Migración transaccional a `user_version=2`:

- `tipos_membresia.duracion_dias`, opcional, de 1 a 3660. `NULL` conserva los meses históricos. Los planes en días usan esta columna; `duracion_meses=1` queda como valor de compatibilidad con la restricción positiva histórica y no se utiliza para calcular esos planes.
- `visitas_ocasionales`: identificador de operación, nombre opcional y fecha/hora local.
- `pagos_visitas`: una fila por visita, importe fijo y fecha real del cobro.
- `asistencias_visitas`: una fila por visita y fecha/hora local.
- Vistas de lectura `ingresos_todos` y `asistencias_todas` para sumar miembros y visitas sin duplicarlos.

Las tablas históricas de clientes, pagos y asistencias no se reconstruyen ni se reescriben. Las visitas tienen relaciones propias y no crean clientes ficticios. El total financiero es membresías + inscripciones + visitas. Las visitas cuentan como entradas, pero no como miembros activos ni huellas.

Una transacción inmediata registra las tres filas de cada visita. El ID aleatorio de 128 bits es único: reenvíos concurrentes devuelven el recibo existente sin otro cobro o asistencia. La interfaz bloquea doble clic, conserva la operación al fallar y guarda únicamente su ID pendiente en el almacenamiento del navegador para recuperarla tras reiniciar. El nombre no se guarda allí. Un reintento de recuperación con nombre vacío no borra el nombre ya persistido.

Se conserva SQLCipher/DPAPI. La migración utiliza el mecanismo de copia cifrada anterior para instalaciones existentes que aún no tengan la versión 2. La restauración reconoce ahora respaldos de versión 2; no cambia el cifrado ni la autenticación. La versión anterior de Logym rechaza una base con versión 2: no hacer downgrade sobre la base migrada. Para revertir se necesita el respaldo previo y el procedimiento de recuperación, teniendo en cuenta los datos posteriores.

La actualización de Dashboard y gráficas usa `logym://datos-actualizados`; no se añade polling. No hay cambios en supervisor, SDK, comparación 1:1 o licenciamiento. No se agregan dependencias.

## Archivos

- Backend: `database.rs`, `visitas.rs`, `membresias.rs`, `repositories.rs`, `services.rs`, `commands.rs`, `main.rs`; `recuperacion.rs` solo adapta la versión máxima del respaldo.
- Interfaz: `VisitaOcasional.vue`, `useVisita.js`, `duracionPlanes.js`, `tauriApi.js`, `useRegistroCliente.js`, `CheckIn.vue`, `Registro.vue`, `AdministracionPrecios.vue`.
- Pruebas: `tests/frontend.test.mjs`; Rust en `src-tauri/src/tests/{visitas,database,membresias,repositories,services,business,recuperacion}.rs`.

## Resultados ejecutados

| Comprobación | Resultado |
| --- | --- |
| `cargo test --locked --offline` | 112 aprobadas, 0 fallidas, 0 ignoradas |
| JavaScript: frontend y activación | 68 aprobadas, 0 fallidas |
| Vue/Vite | PASS |
| `scripts/verify-test-isolation.mjs` | PASS, 26 archivos del build |
| `git diff --check HEAD` | PASS |
| Navegador con IPC simulado | PASS: modal, teclado, doble clic, reintento, diseño móvil, meses/días y Finanzas |

Las pruebas usan bases en memoria o temporales y datos sintéticos; las pruebas biométricas usan SDK simulado. Se prueban SQLCipher/DPAPI y reapertura, migración desde versión 1, concurrencia de visitas, rollback por fallo de pago o asistencia, periodos financieros, ausencia de clientes ficticios, alta semanal con/sin huella, cobros, vigencias y renovaciones, y regresiones mensuales. La primera ejecución Rust falló en una fixture financiera antigua que creaba un esquema parcial; se sustituyó por la fixture del esquema real y la suite completa pasó.

El harness de navegador y las capturas están en `.artifacts/visitas-semanal/`, excluido de Git. Este harness usa fronteras IPC simuladas y no inicia el backend de Logym. Se corrigieron esperas y montaje del propio harness antes del resultado final.

## Validación manual pendiente

1. En una instalación y base de prueba, verificar la copia previa, migración, reapertura y recuperación del respaldo con la versión nueva. Nunca usar una base del gimnasio para esta aceptación.
2. Crear Semanal de 7 días con precio de prueba; comprobar su selección en registro con/sin huella y renovaciones, y su presentación en Clientes y Dashboard.
3. Confirmar vigencia en el último día y rechazo después; probar renovación vigente/vencida, inscripción opcional y continuidad de planes mensuales.
4. Registrar visitas con/sin nombre, repetir una confirmación tras fallo de respuesta y comprobar exactamente una entrada/cobro; registrar otra entrada deliberada y comprobar un nuevo cobro.
5. Verificar actualizaciones visibles de Dashboard y Finanzas con datos de prueba, periodos y categorías combinadas; repetir con Internet desconectado.
6. Probar teclado, ventanas mínimas y notificaciones junto con lecturas reales del SecuGen. No se ha realizado prueba física del lector ni instalación limpia en esta revisión.

Riesgos antes de entregar: validar la migración y recuperación en una copia de prueba, funcionamiento completo en Windows/Tauri y la operación física. Al no identificarse biométricamente a visitantes ocasionales, evitar reingresos depende del control del operador: no hay una identidad de visitante que permita detectar por software a quien vuelve. No se declara lista para producción hasta la aceptación manual. El instalador generado anteriormente no contiene estos cambios sin publicar.
