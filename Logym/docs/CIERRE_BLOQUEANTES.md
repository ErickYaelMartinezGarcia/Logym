# Cierre de bloqueantes: SMBIOS/CIM, bloqueos y recuperación

Se conservan Ed25519, la clave pública, LOGYM-HARDWARE-V2, la serialización JSON
canónica y el matcher SecuGen SG400/400 bytes/nivel 3. No hay formulario manual
de hardware ni identidad persistida alternativa. No se usan bases reales.

## HP y compatibilidad

En la HP ProBook 4530s real se comprobó SMBIOS 2.6, Type 1 de 27 bytes y Type 2
de 16 bytes con BoardType=1 (Unknown). La condición anterior `resto[13] != 10`
rechazaba esa placa aun teniendo identificadores válidos. Ahora una placa única
con datos completos no depende de ese campo opcional. También se admite Type 2
de ocho bytes; con varias placas se exige una principal inequívoca.

CIM es la segunda vía automática local: ComputerSystemProduct.UUID y los campos
Manufacturer/Product/SerialNumber de BaseBoard. No acepta datos del usuario ni
perfiles PowerShell o consultas interpoladas. La salida está limitada y tiene
timeout de diez segundos. Ambas vías usan la misma normalización y hash.
SMBIOS anterior a 2.6 pasa a CIM para no adivinar el orden UUID antiguo. Si ninguna
vía es fiable se rechaza la activación; no se acepta una identidad en caché.

El digest literal de la identidad sintética firmada antes del cambio se conserva.
En la HP real ambas vías coincidieron con el hash de los datos aportados por el
propietario. Dos procesos release independientes mantuvieron el código. Se emitió
con la clave existente una licencia pública de prueba y otros dos procesos nuevos
verificaron firma e identidad; una licencia para otra PC fue rechazada. No se tocó
la activación instalada. Evidencia local: `.artifacts/cierre/`.

## Orden de bloqueos y unicidad

Antes, identificar_template adquiría GLOBAL_MANAGER y luego consultaba SQLite;
guardar_cliente_sin_duplicar adquiría SQLite Immediate y luego GLOBAL_MANAGER.
Eso permite el ciclo SQLite -> SDK -> SQLite.

Ahora identificar_template materializa la lista y destruye filas/statement ANTES
de adquirir GLOBAL_MANAGER. Bajo el mutex SDK no queda acceso a SQLite. El alta
conserva SQLite -> SDK y su transacción incluye comprobación, cliente, pago e
historial. El worker libera el SDK antes de consultar expediente o registrar
asistencia. No queda una arista SDK -> SQLite que cierre el ciclo. Conservar una
conexión abierta no equivale a mantener un bloqueo de lectura.

Una prueba usa SQLite temporal en modo DELETE, transacción EXCLUSIVE y un
busy_handler: mientras identificar espera SQLite, otro hilo debe poder comparar
en el SDK. Otra sincroniza cuatro altas y cuatro check-ins sobre WAL y el
controlador real con ABI SDK simulada: exige completar con timeout, una sola
alta/cobro y una sola asistencia. No certifica la respuesta física del driver USB.

El matcher productivo sigue siendo SGFPM_MatchTemplate. Inactivos, SDK con error y
plantillas dañadas no permiten clientes, pagos ni historial parciales. La igualdad
de bytes pertenece únicamente a la simulación del SDK, nunca al matcher productivo.

## Recuperación y licencia

La prueba completa usa SQLCipher y DPAPI temporales. Compara cada columna y fila
de TODAS las tablas, incluyendo fotos, huellas, administrador, planes, pagos
históricos, membresías y asistencias. Prueba destino nuevo con otra clave local,
reemplazo con snapshot previo válido, integridad, FK, contraseñas incorrectas,
archivo corrupto, FK huérfanas y versión futura. Ante rechazo el destino conserva
sus registros y license.txt permanece intacto. El respaldo no transfiere licencia.

Las operaciones sensibles siguen verificando licencia en Rust: altas, edición,
bajas, precios, pagos/informes, membresías, asistencias y biometría. Estado y
solicitud son públicos para permitir activar. Mantenimiento conserva su control
administrativo independiente; no permite operar el gimnasio sin licencia. El
bypass debug preexistente no se utiliza para aceptar release.

## Evidencia y aceptación pendiente

Se conservan todas las pruebas anteriores. Ocho pruebas nuevas cubren estos
escenarios sin sustituir la suite por pruebas redundantes. La evidencia y el
manifiesto de commit, inputs, ejecutable e instalador están en `.artifacts/cierre/`
(ignorada). No se publican licencias locales ni UUID/serial reales en Git.

Pendientes: instalación/actualización visual de este NSIS, reinicio de la GUI
instalada, SecuGen físico (captura, comparación y reconexión), restauración en otro
equipo/perfil y reinstalación/cambio de SSD físicos. No se creó otro usuario Windows
ni se trabajó con bases reales. Las simulaciones no certifican aceptación física.
Un administrador o hipervisor puede falsificar SMBIOS/WMI: no se promete atestación
de hardware ni detección de identificadores OEM duplicados de aspecto válido.

Fuentes: [SMBIOS Windows](https://learn.microsoft.com/en-us/windows-hardware/drivers/bringup/smbios),
[Win32_BaseBoard](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-baseboard).
