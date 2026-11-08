# Licencia offline vinculada al hardware (LOGYM2-HW)

La solicitud `LOGYM2-HW-<64 hex>` contiene SHA-256 de una representación JSON
versionada de UUID SMBIOS, fabricante, modelo y número de serie de placa base.
Ed25519 firma la solicitud completa. La clave pública de producción no cambia.
La clave privada sigue exclusivamente fuera del proyecto; nunca se distribuye
con la aplicación, la base, las pruebas ni el instalador.

La vía principal obtiene SMBIOS mediante `GetSystemFirmwareTable('RSMB')`.
Se exige UUID completo y placa identificable; Type 2 admite la longitud mínima
de ocho bytes y no exige BoardType=10 cuando la única placa OEM no lo informa.
Si hay varias, se exige una única placa principal (HostingBoard/BoardType).
Si falla la vía directa, Windows CIM obtiene automáticamente
Win32_ComputerSystemProduct.UUID y Win32_BaseBoard. No hay formulario manual.
En SMBIOS anterior a 2.6 se usa CIM para no adivinar el orden UUID antiguo.
Ambas vías usan la MISMA serialización canónica anterior, sin cambiar LOGYM2-HW.
El UUID se convierte del orden SMBIOS little-endian a su orden canónico.
Las cadenas ASCII se recortan, se reducen espacios y se convierten a mayúsculas.
No se usan versión de BIOS, SSD, volumen, MAC, usuario, MachineGuid ni DPAPI.
Dos lecturas consecutivas de la vía elegida deben producir la misma identidad.
Una lectura SMBIOS inconsistente no se oculta con un fallback. No se guarda una
identidad en disco ni se acepta una identidad en caché cuando fallan las lecturas.
Cada comprobación vuelve a obtener los datos de Windows. Los errores no se ocultan.
La consulta CIM usa PowerShell de System32, sin perfil ni ventana, con script
constante (sin interpolar entradas), salida limitada y timeout de diez segundos.
Requiere el proveedor CIM local y Windows PowerShell de Windows 10/11; no requiere
Internet ni elevación. Si ambas vías fallan, se rechaza la activación.

UUID vacío, uniforme o con patrones genéricos conocidos, serial vacío, genérico,
insuficiente, tablas truncadas, índices incorrectos y estructuras ambiguas
producen un error visible. No hay fallback aleatorio ni identidad universal.
No se registran UUID/serial en logs ni se envían a un servidor. La solicitud es
un identificador persistente: tratarla como información del equipo.

## Pantalla de activación

Al arrancar, la interfaz consulta la licencia antes de montar las pantallas del
sistema. Sin licencia válida muestra automáticamente el código de hardware,
Copiar código, el campo Licencia firmada y Activar licencia. Un rechazo de firma
o de computadora se muestra en esa pantalla y no abre las vistas de datos.
Tras activar se verifica nuevamente la licencia y se entra sin reiniciar.
Con licencia válida se inicia directamente. En Información del sistema, el punto
verde junto al nombre abre la consulta de licencia, sin un botón público en el
menú. Es un acceso discreto, no autenticación exclusiva de desarrollador.
La consulta permite ver el estado y copiar el código; Volver al sistema
cierra esa consulta sin volver a montar las vistas. Si WebView no permite el
portapapeles moderno, se intenta copiar la selección y se informa cómo hacerlo
manualmente si también falla.

Siete pruebas nuevas montan los componentes Vue de producción con un renderer
de pruebas y fronteras IPC simuladas: arranque bloqueado sin llamadas de datos,
activación sin reinicio, licencia inválida/ajena, arranque ya activado y consulta,
errores IPC/SMBIOS, copia y fallback. La aceptación visual final del instalador
sobre WebView2 y hardware del gimnasio sigue siendo una prueba física.

## Activar y actualizar

Enviar al emisor el código nuevo mostrado en Activación. En la PC emisora:

```powershell
node tools/licencias.mjs firmar 'RUTA_PRIVADA_FUERA_DEL_PROYECTO' 'LOGYM2-HW-codigo_de_la_app'
```

Pegar la firma devuelta. El emisor comprueba que la clave privada corresponde a
la pública configurada. La aplicación verifica la firma y vuelve a obtener el
hardware local al validar. Copiar `license.txt` y una base a otra computadora con
UUID/placa distintos no autoriza esa computadora.

Las licencias LOGYM1 anteriores se rechazan explícitamente. Es necesario emitir
una nueva licencia de hardware con la MISMA pareja Ed25519. No se convierten
silenciosamente ni se eliminan. `installation.dpapi` deja de usarse y no se borra.
Un error de hardware/licencia bloquea operaciones licenciadas pero no abre,
modifica, migra ni elimina SQLCipher o su clave local. El mantenimiento de
recuperación conserva su autenticación administrativa y no depende de activación.

Reinstalar Windows o sustituir el SSD conserva la identidad si el firmware
reporta exactamente el mismo UUID y datos de placa normalizados. Conservar una
copia de `license.txt` y volver a aplicarla después de reinstalar. Esto NO vuelve
portable `gym.key.dpapi`: para recuperar la base tras reinstalar Windows hace
falta un respaldo portable con contraseña y restaurarlo con el procedimiento
existente. Conservar solo gym.db y su archivo DPAPI antiguo no es suficiente.

## Limitaciones que deben aceptarse antes de publicar

- SMBIOS no es una raíz de confianza criptográfica. Un administrador, firmware
  modificado o hipervisor puede falsificarlo; una VM clonada que conserva todos
  los identificadores puede compartir licencia. Tampoco se impide parchear el
  ejecutable. No se promete resistencia frente al dueño hostil del sistema.
- No es posible demostrar offline que todos los fabricantes asignaron UUID y
  serial únicos. Se rechazan patrones conocidos, pero duplicados OEM de aspecto
  válido e idénticos en todos los campos siguen siendo indistinguibles. Un hash
  no añade autenticidad a identificadores mal aprovisionados.
- Cambio de placa, reparación OEM, reprovisionamiento o actualización de firmware
  que altere esos campos exige reemisión. Dos lecturas estables no garantizan
  que un firmware futuro conservará los valores.
- Equipos sin una placa inequívoca, serial genérico/ausente o caracteres no ASCII
  quedan bloqueados si tampoco CIM puede proporcionar datos válidos. Corregir
  el aprovisionamiento con el fabricante; no usar un código común como remedio.
- El bypass de desarrollo preexistente sigue limitado a debug. La aceptación
  comercial debe hacerse sobre release; no distribuir builds de desarrollo.
- Faltan ensayos físicos de esta versión en Windows 10/11: lectura como operador,
  activación y reapertura, reinstalación real, cambio de SSD y copia a otra PC.
  Las simulaciones no certifican la estabilidad del firmware del gimnasio.

Pruebas automatizadas: firma del emisor real sobre hardware sintético, activación
y reapertura, sustitución de disco, reinstalación sin installation.dpapi, rechazo
en otra PC y de LOGYM1, normalización, firmware truncado/genérico/ambiguo y
lecturas inconsistentes. Solo se usan datos sintéticos. La fixture guarda una
firma pública; no contiene claves privadas.

Fuentes: [API de firmware Windows](https://learn.microsoft.com/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemfirmwaretable),
[especificación SMBIOS DMTF DSP0134](https://www.dmtf.org/sites/default/files/standards/documents/DSP0134_3.8.0.pdf).

Diagnóstico de solo lectura desde el release (no abre SQLCipher):

```powershell
& 'C:\ruta\VIRTUS GYM.exe' --diagnostico-licencia
& 'C:\ruta\VIRTUS GYM.exe' --diagnostico-licencia 'C:\ruta\license.txt'
```

Devuelve códigos hash/resultados de ambas vías y, opcionalmente, verifica una
licencia con la identidad leída en ese nuevo proceso. No admite identificadores
introducidos por el usuario ni escribe activaciones. En diagnóstico conviene
redirigir stdout o invocarlo desde un proceso con stdout capturado.
