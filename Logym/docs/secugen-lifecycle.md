# SecuGen: ciclo de vida y evidencia de recuperación

Referencias locales: `FDx SDK Pro for Windows v4.3.1`.

- `Documents/How to Develop Applications Compatible with All SecuGen USB Readers
  (SG1-0030D-001).pdf`, pp. 2-4: Create, EnumerateDevice, Init con DevName
  enumerado, OpenDevice, GetDeviceInfo, captura, CloseDevice, Terminate.
- `Samples/VisualC++/DeviceTest/sgdvcDlg.cpp`, líneas 305-315: enumerar antes
  de Init está permitido; líneas 414-416: inicializar el modelo enumerado.
- `Inc/sgfplib.h`, líneas 421-429: firma C de EnumerateDevice con tres argumentos;
  Terminate recibe el handle, no un puntero al handle. Línea 77: USB_AUTO_DETECT=957.
- `Documents/FDx SDK Pro Programming Manual (SG1-0030A-026).pdf`, p. 33:
  GetDeviceInfo puede devolver información del driver antes de inicializar el
  dispositivo. Pp. 35-36: timeout no equivale a captura válida. Pp. 7-8 y 29-30:
  Init carga el driver; Terminate libera el objeto y debe llamarse antes de salir.

## Carga de DLL

Se cargan `sgfplib.dll` y el algoritmo oficial `sgfpamx.dll` por ruta absoluta,
con búsqueda de sus dependencias en su propio directorio y System32. El módulo
del algoritmo está identificado por el manual nativo, apartado 2.2. En el binario
x64 local, la referencia a `sgfpaMX.dll` en 0x180019ba1 llega a LoadLibraryA
en 0x18001d843. Retener esa biblioteca evita depender de la búsqueda global
para el algoritmo. No se altera PATH ni se llama a SetDllDirectory o
SetDefaultDllDirectories. Los drivers de los modelos los resuelve el SDK
mediante la instalación oficial; no se precargan nombres deducidos.

Inspección de las tablas PE de los binarios locales (x64, máquina 0x8664):

| DLL | Importaciones estáticas |
| --- | --- |
| sgfplib.dll | sgwsqlib.dll, VERSION.dll, WTSAPI32.dll, KERNEL32.dll, USER32.dll, ADVAPI32.dll, SETUPAPI.dll |
| sgfpamx.dll | KERNEL32.dll |
| sgwsqlib.dll | VCRUNTIME140.dll, KERNEL32.dll y API sets de Universal CRT: heap, stdio, convert, string, runtime, math |

Estas tres DLL no tienen tabla de importaciones diferidas. Las importaciones
estáticas no identifican los plugins que el SDK pueda cargar dinámicamente.
Error 6 significa DLLLOAD_FAILED_DRV: no demuestra qué archivo falló, una
arquitectura incorrecta o ausencia del driver. El diagnóstico incluye modelo
enumerado, arquitectura del proceso y ruta del SDK. Para demostrar la causa
en el equipo afectado, revisar la instalación oficial y registrar las cargas
fallidas del proceso, por ejemplo con Process Monitor.

## Estados y recuperación

PnP se consulta en un hilo independiente cada 500 ms, sin llamadas al SDK.
Se observa el VID SecuGen del INF instalado (1162), sin fijar un solo PID.
Un cambio de lista incrementa la generación y descarta evidencia anterior.
El monitor actualiza el estado y sus indicadores de evidencia bajo el mismo
mutex: ninguna consulta puede observar Disponible con la captura invalidada
durante una retirada o reinserción USB.
La enumeración del SDK decide el modelo. Se exige un solo lector: así
USB_AUTO_DETECT no puede seleccionar un dispositivo distinto del enumerado.

El estado consultable y los eventos distinguen USB presente, SDK inicializado
y captura comprobada. La alerta permanece hasta GetImageEx=0 en el manager y
generación actuales. GetDeviceInfo=0 y GetImageEx=54 no la retiran. Después de
una captura comprobada, un timeout normal conserva la evidencia de esa sesión.
Una lectura parcial, errónea o rechazada (53/57/62) permite reintentar sin el
backoff de recuperación; tampoco lo aplican 2/101/105 de CreateTemplate.

Fallos persistentes o de recuperación esperan cinco segundos desde el final
del intento. Después de tres fallos consecutivos, el manager se invalida y el
siguiente intento lo destruye antes de crear otro. Cambiar la generación USB
adelanta el siguiente intento. No se recupera desde el worker mientras esté
pausado para registrar clientes.

Todas las llamadas nativas, incluido CloseDevice/Terminate, se serializan bajo
GLOBAL_MANAGER. El cierre de Tauri solicita parar, previene la salida inmediata
y delega la espera de los hilos y la destrucción del manager a un coordinador.
El hilo de eventos no espera el mutex ni hace join. Las nuevas operaciones
rechazan el estado de cierre. Un driver que se bloquee dentro de una llamada
nativa puede demorar la salida; no se fuerza la liberación de un handle en uso.
El coordinador toma los JoinHandle y suelta el mutex de la lista antes de
esperarlos. Solo después libera el manager. En Tauri 1.8.3, AppHandle::exit
ejecuta cleanup_before_exit y std::process::exit; no vuelve a solicitar
ExitRequested. CIERRE impide lanzar coordinadores duplicados.

## Validación física pendiente

Probar en Windows: arranque con USB conectado, retiro en Dashboard durante el
timeout de 4000 ms, reinserción, huella parcial, huella válida y cierre durante
captura. La alerta debe aparecer al retirar el USB y permanecer tras reabrir
el SDK hasta capturar una imagen válida. Comprobar recuperación tras varios
Init:6 y revisar errores CloseDevice/Terminate. Las pruebas unitarias usan un
SDK simulado: no certifican el funcionamiento del hardware ni de su driver.

El matcher 1:1, el nivel 3, SG400, las plantillas de 400 bytes y la lógica de
asistencia se conservan.