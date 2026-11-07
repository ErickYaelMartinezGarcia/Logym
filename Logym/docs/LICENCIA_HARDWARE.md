# Licencia offline vinculada al hardware (LOGYM2-HW)

La solicitud `LOGYM2-HW-<64 hex>` contiene SHA-256 de una representación JSON
versionada de UUID SMBIOS, fabricante, modelo y número de serie de placa base.
Ed25519 firma la solicitud completa. La clave pública de producción no cambia.
La clave privada sigue exclusivamente fuera del proyecto; nunca se distribuye
con la aplicación, la base, las pruebas ni el instalador.

Windows obtiene SMBIOS mediante `GetSystemFirmwareTable('RSMB')`, sin WMI,
PowerShell, Internet, privilegios administrativos ni servicios de terceros.
Se exige SMBIOS >= 2.6, una estructura System Information (tipo 1) y una sola
Baseboard Information (tipo 2) identificada como motherboard (tipo 10).
El UUID se convierte del orden SMBIOS little-endian a su orden canónico.
Las cadenas ASCII se recortan, se reducen espacios y se convierten a mayúsculas.
No se usan versión de BIOS, SSD, volumen, MAC, usuario, MachineGuid ni DPAPI.
Dos lecturas consecutivas deben producir la misma identidad. No se guarda una
identidad en caché que permita aceptar otro equipo o disimular errores de lectura.

UUID vacío, uniforme o con patrones genéricos conocidos, serial vacío, genérico,
insuficiente, tablas truncadas, índices incorrectos y estructuras ambiguas
producen un error visible. No hay fallback aleatorio ni identidad universal.
No se registran UUID/serial en logs ni se envían a un servidor. La solicitud es
un identificador persistente: tratarla como información del equipo.

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
- Equipos con SMBIOS anterior a 2.6, múltiples placas, serial genérico/ausente,
  caracteres no ASCII o motherboard sin tipo válido quedan bloqueados. Corregir
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
