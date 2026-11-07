# Entrega de producción: estado y operación

La revisión SMBIOS/CIM, concurrencia y restauración completa se documenta en
[CIERRE_BLOQUEANTES.md](CIERRE_BLOQUEANTES.md). Sus comprobaciones y el manifiesto
del instalador actual están en `.artifacts/cierre/`; las cifras y hashes de
entregas anteriores que aparecen abajo son históricos.

Esta entrega NO certifica instalación limpia ni funcionamiento físico del lector.
Las comprobaciones del equipo destino listadas al final son obligatorias antes
de poner el gimnasio en operación. No se usaron datos reales en las pruebas.

## Identidad de hardware LOGYM2-HW

La solicitud actual es `LOGYM2-HW-<SHA-256>` y firma UUID SMBIOS y datos de
placa normalizados, con la misma clave Ed25519. LOGYM1 requiere reemisión.
Consultar [procedimiento y limitaciones](LICENCIA_HARDWARE.md) antes de publicar.
Validación del cambio: **83 Rust + 30 JavaScript PASS, 0 fallos y 0
ignoradas**. También pasaron build frontend, cargo check/test/build release
con --locked, aislamiento del release y recuperación con bases sintéticas.
Evidencia: `.artifacts/hardware-license/validated/` y
`.artifacts/hardware-license/maintenance-cli-results.json`.
El instalador y los resultados históricos documentados abajo corresponden al
commit `4d4d3b5`: **ese instalador no incluye la identidad de hardware nueva**.
Para esta revisión se debe reconstruir NSIS desde el commit publicado y
comprobar el ejecutable empaquetado. El manifiesto local de correspondencia
queda en `.artifacts/hardware-license/publicacion.json`; los hashes de la
entrega actual se facilitan junto con el instalador.

## Cambios y verificaciones

- SQLCipher conserva la clave aleatoria local protegida por DPAPI y rechaza una
  base existente si falta su clave. La apertura inicial se serializa y la clave
  temporal se reserva sin sobrescribir y se sincroniza antes de renombrarla.
- Migraciones transaccionales y `user_version=1`; se rechazan versiones futuras.
  Antes de migrar una base cifrada existente se exporta un snapshot cifrado con
  su clave local. No se copian archivos de una base activa omitiendo el WAL.
- D2 corregido: agregar columnas no usa defaults no constantes. Las fechas
  históricas ausentes quedan NULL, visibles como no configuradas; no conceden
  acceso ni se inventa vigencia. D3 corregido: el esquema legacy simple de
  asistencias se reconstruye con FK/cascada preservando filas, índices y triggers.
  Si hay huérfanos, columnas/restricciones no reconocidas, vistas o tablas
  dependientes, se revierte la migración sin eliminar datos. Requiere revisión
  asistida de una copia; no se intenta adivinar cómo reparar registros.
- Alta biométrica: la comparación con todas las plantillas, incluidas las de
  clientes inactivos, y el alta comparten transacción Immediate. Se utiliza
  `comparar_templates`, SG400, 400 bytes y nivel 3 sin cambios. Un error del SDK o
  una plantilla almacenada dañada impide el alta. Se rechazan coincidencias sin
  dejar pagos ni historial parcial. La operación se ejecuta fuera del hilo UI.
- La transacción y las fechas de renovación aprobadas se conservan.
- Licencia: nuevo emisor Ed25519 real, verificación estricta de firma y rechazo
  de claves débiles. El build release rechaza una clave pública vacía/cero.
  La identidad aleatoria de instalación continúa protegida por DPAPI. No se
  introdujo bypass de activación en release.

Las 99 pruebas anteriores se conservaron, incluyendo D2/D3 con sus aserciones
originales; estas dos ya pasan y dejaron de estar ignoradas. Se añadieron siete
pruebas Rust: recuperación/importación/migración (4), licencia con firma del
emisor real sobre una identidad sintética (1), alta duplicada con SDK simulado
(1) y altas concurrentes (1). Resultado: **76 Rust + 30 JavaScript aprobadas,
0 fallos y 0 ignoradas**, 106 en total.

Pasaron `npm test`, `npm run build` (51 módulos), `cargo check --locked`,
`cargo test --locked`, `cargo build --release --locked` y
`node scripts/verify-test-isolation.mjs --release`. Se ejecutaron fuera del
sandbox para usar el perfil DPAPI de Windows, exclusivamente con bases
temporales. Logs: `.artifacts/final/validated/` y `.artifacts/tests/latest/`.
También pasó `npm run test:defects`: repite la suite y ejecuta explícitamente
los dos casos D2/D3 corregidos, ambos PASS (ya no usa `--ignored`).

Además pasó `node tests/maintenance-release.cjs`: invoca el ejecutable release
en procesos separados, importa una base sintética, crea y reabre un respaldo,
rechaza credenciales incorrectas y restaura en otro destino con una clave local
nueva. También restaura sobre un destino válido y rechaza importar sobre datos
existentes. Evidencia: `.artifacts/production/maintenance-cli-results.json`.
Esto no equivale a restaurar en otro usuario, equipo o VM.

## Respaldar y restaurar

Cerrar Logym primero. Todas las instancias de esta versión y el mantenimiento
comparten un mutex Windows; no se puede iniciar otra simultáneamente. No usar
una versión anterior de Logym ni otra herramienta SQLite a la vez.

El directorio de datos sigue siendo `%APPDATA%\com.virtusgym.desktop`, fuera de
la carpeta de instalación. La instalación/actualización no debe cambiar este
identificador, usuario Windows ni ubicación sin migración explícita.

Ejemplo PowerShell (sustituir la ruta del ejecutable instalado y del disco de
respaldo):

```powershell
$exe = 'C:\ruta\de\instalacion\logym.exe'
$base = Join-Path $env:APPDATA 'com.virtusgym.desktop\gym.db'
.\tools\Mantenimiento.ps1 -Ejecutable $exe -Operacion respaldar `
  -Base $base -Respaldo 'E:\Respaldos\gym-2026-11-06.sqlcipher'
```

Se solicitan la contraseña administrativa y una contraseña de respaldo de al
menos 16 caracteres. Las contraseñas se envían por stdin, no por argumentos ni
archivos temporales. El respaldo portable usa SQLCipher con esa contraseña,
independiente de DPAPI y de la clave local; debe custodiarse separadamente.
Incluye el catálogo, administrador, pagos, historial, clientes y biometría.
No incluye la identidad ni la licencia de la instalación.

Para un ensayo, crear primero un directorio vacío y usar una base de destino
nueva, jamás la base del gimnasio como prueba:

```powershell
.\tools\Mantenimiento.ps1 -Ejecutable $exe -Operacion restaurar `
  -Base 'C:\RecuperacionValidada\gym.db' `
  -Respaldo 'E:\Respaldos\gym-2026-11-06.sqlcipher'
```

El directorio destino debe existir. Se verifica integridad y autenticación antes
de restaurar, se exporta a un archivo cifrado temporal con una clave DPAPI local,
se valida/migra y se publica mediante reemplazo Windows con WRITE_THROUGH.
Al sobrescribir una base válida, también se exige su contraseña administrativa
y se conserva un snapshot cifrado `pre-restauracion-*.sqlcipher`. La contraseña
administrativa del respaldo y del destino debe coincidir; si cambió, restaurar
primero en otro destino y realizar recuperación asistida. Un destino corrupto
o sin clave no se sobrescribe automáticamente: recuperar en un directorio nuevo,
validar, y planificar el cambio con la aplicación cerrada preservando el original.

Los snapshots automáticos locales requieren el `gym.key.dpapi` original y su
perfil Windows; no sustituyen los respaldos portables con contraseña. No se
eliminan automáticamente respaldos ni se configura una retención silenciosa.
El responsable debe guardar una copia portable diaria fuera del disco del equipo
y verificar regularmente su restauración. No perder la contraseña del respaldo.
Para medios de solo lectura, copiar el respaldo ya finalizado a una carpeta local
antes de restaurar; SQLCipher necesita abrirlo READ_WRITE para exportar mediante
ATTACH, aunque no modifica su contenido lógico.

Una base SQLite antigua puede importarse a un destino nuevo:

```powershell
.\tools\Mantenimiento.ps1 -Ejecutable $exe -Operacion importar_legacy `
  -Base 'C:\RecuperacionValidada\gym.db' -Respaldo 'E:\Origen\legacy.db'
```

No modifica registros del origen ni sobrescribe el destino existente. Si el
legacy no tenía administrador, se configura la contraseña solicitada solo en la
base cifrada nueva. Primero ensayar sobre una copia sintética o anonimizada.

## Emisión y custodia de licencias

La clave privada se guarda fuera del repositorio y del proyecto; la ruta se
entrega directamente al propietario. Solo se incorporó la clave pública en
`src-tauri/src/license_public_key.hex`. No hay clave privada en binarios,
fixtures, parches ni documentación. La fixture contiene únicamente una firma
pública sobre una identidad sintética que no corresponde a una instalación real.

En la PC emisora, con Node instalado:

```powershell
node tools/licencias.mjs firmar 'RUTA_PRIVADA_FUERA_DEL_PROYECTO' 'LOGYM2-HW-codigo_mostrado_por_la_app'
```

Pegar la licencia resultante en la pantalla de activación del equipo destino.
No regenerar la pareja de claves al actualizar: eso invalidaría licencias
emitidas. El emisor comprueba que su clave corresponde a la pública configurada.
Respaldar la clave privada en dos medios externos cifrados (por ejemplo BitLocker),
separados del equipo y del repositorio, y custodiar sus claves de recuperación.
Verificar que la copia permita emitir una firma que valide contra la misma clave
pública. No enviar la clave privada por chat, correo ni incluirla en el kit.
La ACL del archivo y carpeta del emisor quedó restringida al propietario Windows.
La copia fuera del equipo y su ensayo de recuperación siguen a cargo del dueño.

## Instalador y controlador

Build x64 NSIS: `npm run tauri build -- --bundles nsis`. Tauri incorpora el
instalador offline WebView2. Los recursos son únicamente `sgfplib.dll`,
`sgfpamx.dll`, `sgwsqlib.dll`, `sgfdusdax64.dll`, `sgbledev.dll`, presentes en
`FDx SDK Pro for Windows v4.3.1/bin/x64`. No se agregaron nombres de driver DLL
inventados ni librerías SecuSearch/32-bit al bundle.

El build NSIS se ejecutó y terminó correctamente. Archivo generado:
`src-tauri/target/release/bundle/nsis/VIRTUS GYM_0.1.0_x64-setup.exe`,
222.473.284 bytes; SHA-256:
`82b4642d114f40b99ea67ba0b2ea0d4c13ce5e88cfeb48ff726e69f1a1063d54`.
Se verificaron las cinco DLL como PE x64 y contra sus hashes en el SDK local;
el script NSIS generado incluye WebView2 offline y únicamente esos recursos.
Evidencia: `.artifacts/production/installer.json` y `sdk-packaging.json`.
No se ejecutó el instalador en este equipo ni en una máquina limpia.
El controlador oficial aún debe incorporarse al kit y validarse en destino.

| Verificación | Resultado |
| --- | --- |
| Pruebas Rust/JS, builds y aislamiento | PASS |
| Mantenimiento del release con bases sintéticas | PASS |
| Generación NSIS x64 con WebView2 offline | PASS |
| Sintaxis de scripts PowerShell y emisor Node | PASS |
| Instalación limpia y actualización del instalador conservando datos | SKIP: falta equipo/VM de aceptación |
| Activación de la instalación real del gimnasio | SKIP: falta solicitud de ese equipo |
| Recuperación en otro equipo/perfil Windows | SKIP: falta equipo/perfil de aceptación |
| Captura, duplicidad y reconexión física SecuGen | SKIP: no se ejecutaron pruebas físicas |

`sgwsqlib.dll` requiere Visual C++ runtime. Preparar el kit con el redistribuible
x64 firmado por Microsoft y el instalador oficial firmado del controlador:

```powershell
.\tools\Preparar-Entrega.ps1 -ControladorOficial 'C:\Descargas\DriverOficial.exe' `
  -RuntimeMicrosoft 'C:\Descargas\vc_redist.x64.exe' `
  -InstaladorLogym 'src-tauri\target\release\bundle\nsis\VIRTUS GYM_0.1.0_x64-setup.exe'
```

El kit queda en `.artifacts/entrega-*`, con manifiesto SHA-256 y scripts de
instalación/mantenimiento. `Instalar-Logym.ps1 -Carpeta RUTA_DEL_KIT` verifica
hashes y firmas de Microsoft/SecuGen, instala el runtime y abre el asistente del
driver; no inventa parámetros silenciosos. La app se instala por usuario para
mantener el mismo perfil DPAPI. Las dependencias solicitan elevación por UAC.
Si se requiere reinicio, hacerlo antes de continuar. Revisar Administrador de
dispositivos y la herramienta oficial SecuGen antes de activar/capturar en Logym.
No usar el paquete WBF/Windows Hello en sustitución del controlador del FDx SDK.
No seleccionar la opción de eliminar datos de la aplicación al desinstalar.
El NSIS estándar la ofrece como opción explícita; la actualización usa su modo
pasivo y no la selecciona. Confirmar este comportamiento en el ensayo real de
actualización y contar con respaldo portable verificado antes de actualizar.

Fuentes: [controladores oficiales SecuGen](https://secugen.com/drivers/),
[redistribuible Microsoft](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist),
[instaladores Tauri v1](https://v1.tauri.app/v1/guides/building/windows/).
No se presume disponibilidad del controlador en la PC nueva por funcionar en la
PC de desarrollo. El instalador de la aplicación no tiene certificado de firma
de código configurado; verificar su hash por un canal confiable antes de aceptar
la advertencia de editor desconocido. El manifiesto no reemplaza una firma.

## Bloqueantes que requieren aceptación física

1. Completar el kit con el controlador oficial validado y probar instalación
   limpia Windows 10/11 x64 sin herramientas de desarrollo y sin conexión.
2. Activar el release con la solicitud real del equipo destino; reabrir y
   confirmar persistencia de licencia y datos con el usuario operador.
3. En una instalación de pruebas: actualizar conservando datos; restaurar desde
   USB y desde otro equipo/perfil Windows; ensayar recuperación de clave local
   perdida y de una base corrupta manteniendo el original.
4. SecuGen físico: alta, mismo dedo en dos altas (también cliente inactivo),
   dedo distinto, check-in, retiro/reinserción USB desde Dashboard, pausa,
   cierre durante captura y nuevo inicio. No se cambió ni certificó el matcher.
5. Si hay datos legacy reales: revisar una copia anonimizada del esquema y los
   registros sin fechas/huérfanos antes de migrar; no probar sobre la base real.
6. Custodia y respaldo externo verificado de la clave del emisor y de un
   respaldo portable. Sin recuperación verificable no autorizar producción.
