# Batería automatizada de Logym

Estado posterior de producción: consultar [ENTREGA_PRODUCCION.md](ENTREGA_PRODUCCION.md).
D2/D3 se corrigieron y sus pruebas originales ya no están ignoradas. La suite
actual tiene 106 pruebas aprobadas (76 Rust y 30 JS). Las cifras y defectos de
las secciones siguientes son evidencia histórica de las entregas anteriores.

## Validación final de fechas almacenadas — 6 de noviembre de 2026

Se conservaron las 96 pruebas anteriores y se añadieron tres en
`src-tauri/src/tests/repositories.rs`: rechazo de fechas presentes corruptas,
renovación fallida sin cambios en clientes/historial/pagos y conservación del
comportamiento con fechas ausentes. Las dos primeras fallaron antes de la
corrección: `periodo_renovacion` devolvía un período válido y la renovación
devolvía `Ok("Membresía renovada")` pese a contener fechas corruptas.

Ahora se validan ambas fechas no vacías antes de calcular o escribir el nuevo
período, incluso el inicio de una membresía vencida. Una fecha inválida devuelve
un error que identifica inicio o vencimiento; la transacción se revierte sin
sobrescribir datos ni agregar historial o pagos. Las fechas vacías conservan el
comportamiento previo. La transacción Immediate conserva la serialización de
renovaciones concurrentes y los días pagados. No se cambiaron precios, lector,
matcher ni migraciones legacy D2/D3, que siguen pendientes e ignoradas.

Resultados reales tras la corrección, fuera del sandbox:

| Comando | Resultado |
| --- | --- |
| npm test | PASS: 67 Rust + 30 JS; 2 ignoradas |
| npm run build | PASS: 51 módulos |
| cargo check --locked | PASS |
| cargo test --locked | PASS: 67 aprobadas, 0 fallos, 2 ignoradas |
| cargo build --release --locked | PASS |
| node scripts/verify-test-isolation.mjs --release | PASS |

Son 99 pruebas distintas: 97 aprobadas y dos defectos conocidos ignorados.
Los resultados históricos siguientes corresponden a la batería de 96 pruebas.
El intento inicial dentro del sandbox falló en tres pruebas por
`Windows DPAPI: El sistema no puede encontrar el archivo especificado. (os error 2)`;
Vite también falló con `Cannot read directory "../../..": Acceso denegado.`.
La repetición fuera del sandbox pasó sin modificar las pruebas para ocultarlo.
Logs y comandos: `.artifacts/final/validated/results.json` y los `.log` vecinos;
el intento en sandbox permanece en `.artifacts/final/`.

El build regeneró los archivos versionados de `dist/`; se conservan y se incluyen
en el parche acumulado. Los módulos de prueba continúan excluidos por cfg(test)
y el auditor del release pasó. No se ejecutó la aplicación ni hardware físico.

Ejecución real: 5 de noviembre de 2026, Windows. Sin conexión al SDK físico.
No se abrió la base de datos del gimnasio ni se utilizó AppHandle para resolver
rutas de datos. No se agregaron dependencias npm o Rust.

## Ejecutar

Desde `Logym` (la carpeta que contiene package.json):

```powershell
npm test
```

Ejecuta cargo check, las pruebas Rust y las pruebas JavaScript con cobertura.
Para incluir explícitamente los defectos conocidos:

```powershell
npm run test:defects
```

Este último comando termina con código 1 mientras las dos pruebas de migración
fallen. No convierte los fallos conocidos en PASS. Por separado:

```powershell
cd src-tauri
cargo check --locked
cargo test --locked
cargo test --locked defecto_ -- --ignored --nocapture
cd ..
node --experimental-vm-modules --experimental-test-coverage --test-coverage-include=src/** --test tests/frontend.test.mjs
npm run build
```

DPAPI debe ejecutarse bajo un usuario Windows con su perfil accesible. En el
sandbox apareció `Windows DPAPI: El sistema no puede encontrar el archivo
especificado. (os error 2)`; las mismas pruebas se ejecutaron correctamente
fuera del sandbox. No es una prueba de descifrado fallida con la clave correcta.
Los módulos VM de Node muestran una advertencia experimental; no son una
dependencia adicional. Se utilizó Node 24.13.0 en esta ejecución.

## Resultados observados

| Suite | PASS | FAIL | SKIP | Salida |
| --- | ---: | ---: | ---: | --- |
| Rust, regresión | 64 | 0 | 2 | 0 |
| JavaScript, lógica Vue | 30 | 0 | 0 | 0 |
| Diagnóstico de migraciones (`--ignored`) | 0 | 2 | 0 | Cargo: 101 |
| cargo check --locked | Correcto | — | — | 0 |
| npm run build | Correcto, 51 módulos | — | — | 0 |

Las dos pruebas SKIP de la suite normal son las mismas dos que fallan al
ejecutarlas explícitamente. Hay 96 pruebas distintas: 94 pasan y dos defectos
permanecen abiertos. No se presenta el proyecto completo como libre de fallos.

El runner guarda comandos, códigos de salida y stdout/stderr en
`.artifacts/tests/latest/results.json` y los archivos `.log` del mismo
directorio. Estos artefactos están excluidos de Git; esta documentación y todo
el código de pruebas sí forman parte de los cambios revisables.

## Separación entre aplicación y pruebas

La limpieza arquitectónica conserva las 96 pruebas. Los 66 nombres de tests
Rust son idénticos antes y después; también se conservaron los cuerpos de los
siete módulos reubicados y el archivo de las 30 pruebas JavaScript. Los dos
`#[ignore]` y sus aserciones de defecto permanecen intactos.

```text
Logym/
  src/                         # Vue y composables productivos, sin imports de tests
  src-tauri/src/
    main.rs                    # Registra módulos exclusivos mediante cfg(test)
    database.rs                # Persistencia real + declaraciones de módulos de test
    services.rs                # Servicios y validaciones reales, sin fixtures
    repositories.rs            # Reglas reales, incluida la renovación corregida
    lector_estado.rs           # Estado real, sin función de simulación USB
    lector_worker.rs           # Worker real
    secugen.rs                 # FFI y ciclo de vida reales, sin mocks
    licencia.rs
    membresias.rs
    tests/
      business.rs
      database.rs
      database_migrations.rs
      lector_estado.rs          # Incluye helper de simulación USB exclusivo de tests
      lector_worker.rs
      licencia.rs
      membresias.rs
      repositories.rs
      secugen.rs                # SDK simulado, fixtures y pruebas biométricas
      services.rs
      test_support.rs           # Bases temporales y datos sintéticos compartidos
  tests/frontend.test.mjs       # Vue real con fronteras simuladas
  scripts/run-tests.mjs
  scripts/verify-test-isolation.mjs
  docs/PRUEBAS_AUTOMATIZADAS.md
  .artifacts/                   # Logs y evidencia local; excluido de Git y del bundle
```

Los módulos externos siguen siendo hijos de su módulo productivo mediante
`#[cfg(test)]` y `#[path = "tests/archivo.rs"]`. Así pueden probar funciones
privadas reales sin ampliar su visibilidad en producción. No hay copia de las
reglas de negocio en los tests ni selección de comportamiento mediante
variables de entorno de prueba.

Archivos productivos modificados **en esta limpieza**, frente al estado de
trabajo recibido (no frente al HEAD que aún incluye cambios anteriores):

| Archivo | Cambio exclusivo de organización |
| --- | --- |
| main.rs | Cambiar la ruta de test_support a tests/test_support.rs, conservando cfg(test) |
| database.rs | Extraer pruebas_migracion a tests/database_migrations.rs |
| repositories.rs | Extraer pruebas a tests/repositories.rs; renovación e historial intactos |
| membresias.rs | Extraer tests a tests/membresias.rs; precios y funciones intactos |
| licencia.rs | Extraer pruebas_automatizadas a tests/licencia.rs |
| lector_worker.rs | Extraer tests a tests/lector_worker.rs |
| lector_estado.rs | Extraer tests y trasladar simular_usb a su módulo de pruebas |
| secugen.rs | Extraer tests, mocks y constante de modelo usada únicamente por tests |

services.rs se revisó y no necesitó cambios en esta limpieza: las validaciones
privadas existentes son utilizadas por la aplicación, y su módulo de tests ya
estaba separado. Se trasladó test_support.rs sin borrar su contenido. No se
modificaron la renovación, el matcher, los umbrales, el ciclo de vida del lector,
las migraciones, Cargo.toml, Cargo.lock ni tauri.conf.json.

Las capturas del estado anterior y la verificación de preservación están en
`.artifacts/architecture/latest/preservation.json`. Los comandos obligatorios
se ejecutaron realmente y sus salidas están en
`.artifacts/architecture/latest/validations.json` y los `.log` de ese directorio.
`npm run test:defects` continúa devolviendo código 1 por los dos defectos legacy.

Validaciones ejecutadas después de reubicar los tests:

| Comando | Resultado real |
| --- | --- |
| npm test | PASS: 64 Rust + 30 JS; 2 SKIP conocidos |
| npm run test:defects | FAIL esperado: los dos defectos legacy siguen fallando; salida 1 |
| npm run build | PASS: 51 módulos |
| cargo check --locked | PASS |
| cargo test --locked | PASS: 64 pruebas, 2 ignoradas |
| cargo build --release --locked | PASS: perfil optimized; 23 min 19 s por compilación nativa |
| node scripts/verify-test-isolation.mjs --release | PASS |

El archivo de dependencias del compilador release no incluye src/tests/ ni
test_support.rs. Los marcadores logym-test- y fixture.sqlite están presentes
en el ejecutable de pruebas como control positivo y ausentes en el ejecutable
release. Se revisaron además los tres assets de Vite y los recursos de Tauri.
La evidencia está en `.artifacts/architecture/latest/test-isolation.json` y
`test-isolation.log`. El release se compiló y se inspeccionó, pero no se ejecutó
la aplicación ni se probó el hardware. Los archivos dist previos se restauraron
después de validar para conservar los cambios locales ajenos a esta tarea.

Para verificar el aislamiento después de construir los assets y el release:

```powershell
npm run build
cd src-tauri
cargo build --release --locked
cd ..
node scripts/verify-test-isolation.mjs --release
```

El auditor comprueba las declaraciones Rust condicionadas, los imports Vue/JS,
los assets en dist, la whitelist Tauri y, con --release, las dependencias de
rustc y marcadores de fixtures en el ejecutable. El SDK simulado, las bases
temporales y los helpers se excluyen por cfg(test). Los tests JS, el runner,
el auditor, la instrumentación de cobertura, los informes y los logs están
fuera de src/ y de dist/. Tauri solo agrega bin/**/* como recursos externos.
La búsqueda de marcadores es una comprobación adicional, no una demostración
general de ausencia de cualquier cadena; las declaraciones cfg y el archivo
de dependencias del compilador justifican la exclusión del código de test.

El auditor es deliberadamente conservador: si se agrega public/ o se cambia
la whitelist de recursos Tauri, exige revisar el nuevo empaquetado. No ejecuta
el lector ni la aplicación, y no constituye una prueba física del hardware.

## Cobertura funcional

| Área | Escenarios ejecutados |
| --- | --- |
| Catálogo | Los cinco planes iniciales: Mensual 1 mes/$350, Pareja 1 mes/$300, Estudiante 1 mes/$280, Bimestral 2 meses/$650 y Semestral 6 meses/$1800; inscripción inicial $150; Pareja solo alta; plan sintético administrativo con límites de importe/duración |
| Calendario | Cada día de 2023 y 2024 para los cinco planes: 3.655 combinaciones; casos con resultado exacto para 28/29/30/31 días, 1900 no bisiesto, 2000 bisiesto, febrero y cambio de año; calendario de renovación y de Vue |
| Renovación | Vencida, activa y futura; extensión desde vencimiento; conservación de inicio/acceso; inscripción cero; plan inactivo/solo alta/ausente; fallos inyectados; varias renovaciones concurrentes |
| Cobros | Importes reales de alta/renovación; modificación de catálogo/nombre/inscripción sin reescribir pagos; suma financiera histórica y rangos inclusivos |
| Clientes | Nombre vacío, espacios, Unicode, control, 120/121 caracteres; teléfonos opcionales, caracteres inválidos y longitud; fechas inválidas; foto inválida o excesiva; plantillas de longitud/hex inválidos; activo/inactivo; edición conserva huella; parámetros SQL |
| Asistencia | Membresía futura, vencida, vigente y límites inclusivos de inicio/fin; cliente ausente/inactivo; duplicado diario; ayer 23:59:59 frente a hoy 00:00:00; ocho solicitudes concurrentes y seis sobre SQLCipher |
| Transacciones | Triggers que abortan historial o pago: ni alta ni renovación dejan escrituras parciales; cascadas en esquema nuevo y restricción de eliminación de planes utilizados |
| Persistencia | Esquema real, reapertura, integridad, FK habilitadas y restricciones CHECK; idempotencia y precios personalizados; migración de catálogo sin solo_alta |
| SQLCipher | Apertura de producción sobre ruta temporal; cipher_version; archivo sin encabezado SQLite plano; consulta sin clave y con clave incorrecta rechazada; clave DPAPI persistida; reapertura y concurrencia; rechazo de clave protegida corrupta; base existente sin key.dpapi intacta |
| Autenticación | Alta administrativa única; contraseñas de longitud inválida; hash Argon2id y sales diferentes; contraseña correcta/incorrecta; cambio de precios sin autenticar denegado; licencia hex inválida y firma falsa rechazada por la clave pública configurada |
| SDK simulado | Create/Enumerate/Init/Open/Close/Terminate; errores y nulos; liberación ante fallo; timeout 54, imagen parcial/rechazada, error de extracción y siguiente captura; 400 bytes, SG400 y nivel 3; matcher 1:1 simulado; plantillas almacenadas inválidas/inactivas; desconexión/generación; pausa, cierre y seis controladores concurrentes con un solo manager |
| Vue | Módulos de producción con Vue ref/watch reales; IPC/cámara/router/lifecycle simulados; fechas, confirmación biométrica, guardas de captura, doble clic, errores, renovación, catálogo solo alta, estado del lector y toasts con reloj simulado |

Los precios mencionados son los iniciales del esquema, no una lectura del
catálogo del gimnasio. Las bases temporales se construyen a partir del esquema
de producción; no se cambia ningún precio real para lograr un PASS.

## Cobertura de código medida

Medición V8 sobre los tres módulos JavaScript cargados por las pruebas:

| Archivo | Líneas | Ramas | Funciones |
| --- | ---: | ---: | ---: |
| src/composables/useRegistroCliente.js | 100% | 98,28% | 100% |
| src/composables/useCheckIn.js | 100% | 97,37% | 100% |
| src/services/toast.js | 100% | 87,50% | 100% |
| Agregado de esos tres archivos | 100% | 96,43% | 100% |

Esto no mide vistas .vue, App.vue, cámara, router, puente IPC real ni todo el
frontend. No hay medición porcentual de líneas/ramas Rust: cargo-llvm-cov y
llvm-tools no están instalados. Las 64 pruebas Rust aprobadas y la matriz
anterior son cobertura de escenarios, no un porcentaje de cobertura de código.

## Defectos con reproducción

### D1: renovación eliminaba días pagados — corregido

Antes de tocar la renovación se ejecutaron tres pruebas que fallaron:
`renovar_activa_conserva_dias_pagados_y_acceso`,
`renovar_futura_conserva_inicio_y_extiende_vencimiento` y
`concurrencia_renovacion_no_deja_pagos_parciales`.

Ejemplo real de la primera ejecución, con fecha de sistema 2026-11-05:

```text
Activa: obtenido (2026-11-05, 2026-12-05)
        esperado (2026-10-26, 2027-01-05)
Futura: obtenido (2026-11-05, 2027-01-05)
        esperado (2026-11-10, 2027-02-10)
Cuatro renovaciones: obtenido 2026-12-05, esperado 2027-04-05
```

El usuario confirmó conservar días pagados. La corrección mantiene el inicio
del cliente y añade los meses al vencimiento aún pagado; para una membresía
vencida comienza hoy. El nuevo período del historial comienza después del
último día pagado. Se usa transacción Immediate para serializar la lectura del
vencimiento y su extensión concurrente. Las tres pruebas ahora pasan. No cambió
el precio ni se vuelve a cobrar inscripción.

### D2: migración de clientes poblados sin fechas — abierto

Prueba: `defecto_migracion_cliente_legacy_sin_columnas_de_fechas`.
Fixture sintética mínima: `clientes(id,nombre)` con una fila. Resultado exacto:

```text
Migración falló: Err("Cannot add a column with non-constant default")
```

La inicialización intenta ALTER TABLE con un default basado en date('now').
No se modificó la migración: elegir fechas para datos históricos exige una
política explícita. No se afirma que esa forma de esquema exista en el gimnasio.

### D3: asistencias legacy sin FK conservan huérfanos — abierto

Prueba: `defecto_migracion_asistencias_legacy_no_debe_dejar_huerfanos`.
Fixture: clientes y asistencias preexistentes sin FK; inicializar esquema;
eliminar el cliente por el repositorio. Resultado exacto:

```text
Asistencia huérfana después de migrar/eliminar
left: 1
right: 0
```

CREATE TABLE IF NOT EXISTS no incorpora la FK a una tabla existente. Se
necesita una migración de reconstrucción/conservación de datos, no alterar
silenciosamente los registros existentes para hacer pasar la prueba.

### D4: plantillas idénticas pueden pertenecer a dos clientes — caracterizado

`huellas_sinteticas_identicas_caracterizan_ausencia_de_unicidad` confirma dos
filas activas con los mismos 400 bytes sintéticos. La prueba pasa porque
documenta el comportamiento; no prueba que esa sea una regla deseada ni que
los bytes sean una huella válida. El matcher simulado comprueba el contrato de
llamadas y tamaños; no permite evaluar falsos positivos ni equivalencia de
plantillas biométricas distintas del mismo dedo. No se cambió el matcher 1:1.

## Cambios de producción y límites

La única corrección funcional es D1, precedida por fallos reproducidos y la
confirmación de la regla por el usuario. Se separaron mecánicamente las
validaciones y el cálculo de fechas de alta en funciones privadas para poder
ejecutar ese mismo código sin crear AppHandle. El resto son pruebas, helpers
cfg(test), scripts y documentación. El SDK real, umbral 3, SG400 y 400 bytes se
conservan. Cargo.lock y las dependencias no se modificaron.

No se ejecutó el bucle completo del worker dentro de una ventana Tauri: se
prueban su pausa/inicialización, contador de recuperación, estado y contratos
del SDK. No se midieron tiempos reales de USB, retardo del driver ni fuga nativa
en el DLL del fabricante. Las pruebas de borde de día insertan timestamps
aislados; no adelantan el reloj del sistema ni cubren todos los cambios de zona
horaria/DST. No se ensayaron cortes de energía, disco lleno o restauración de
respaldo. No se prueba la activación release con una licencia válida del emisor.

## Intervención humana mínima pendiente

1. SecuGen real: dos capturas, huella repetida entre clientes, check-in 1:1,
   USB fuera/dentro desde Dashboard, pausa de registro y cierre durante captura.
2. Recorrido completo de la UI en Tauri: alta, renovación de activa/futura,
   doble clic y alerta persistente; comprobarlo también en un build release.
3. Licencia válida del emisor y DPAPI entre usuarios/equipos Windows: rechazo
   donde corresponda y procedimiento de respaldo/restauración autorizado.
4. Revisar una copia anonimizada de los esquemas legacy soportados y decidir
   la migración de D2/D3 antes de desplegar; jamás usar la base real para tests.
5. Check-in a medianoche y cambios de zona horaria/DST en una instalación de
   pruebas, sin alterar el reloj ni los datos del gimnasio.
