# Validación de Logym en Windows

## Alcance y restricciones

- La identificación biométrica **1:1 en bucle** se mantiene deliberadamente; no reemplazar por 1:N.
- El check-in usa sondeo de capturas, **no Auto On nativo**. El SDK requiere el lector físico para pruebas de integración.
- Los datos del cliente (incluida la plantilla de huella) se guardan en SQLite local. El equipo debe protegerse con permisos y cifrado del disco.
- No publicar `gym.db`, plantillas, fotografías ni logs con datos de clientes en GitHub.

## Preparación

1. Confirmar arquitectura de Windows con `(Get-CimInstance Win32_OperatingSystem).OSArchitecture`. Ejecutable y DLL SecuGen deben coincidir.
2. Cerrar Logym antes de respaldar la base de datos. Ubicación habitual: `%APPDATA%\com.virtusgym.desktop\gym.db`. Si hay archivos `gym.db-wal` o `gym.db-shm`, respaldarlos juntos cuando corresponda.
3. Desde la raíz del proyecto ejecutar:
   ```powershell
   git checkout fix/develop-compilacion-tauri
   git pull origin fix/develop-compilacion-tauri
   npm install
   npm run build
   npm run tauri dev
   ```
4. Verificar que el motor inicialice, que SQLite abra y que no haya errores de Rust o JavaScript.

## Matriz manual de aceptación

| Caso | Resultado esperado |
| --- | --- |
| Alta de cliente con huella | Se guarda una sola ficha con su fotografía y plantilla |
| Lectura de huella registrada | Se identifica al cliente correcto sin modificar el algoritmo 1:1 |
| Cliente activo y membresía vigente | Se registra una asistencia |
| Segunda entrada del mismo día | No se duplica la asistencia |
| Cliente desactivado | No aparece en coincidencias biométricas ni puede registrar asistencia |
| Cliente reactivado | Vuelve a poder identificarse |
| Membresía vencida | No se registra entrada; se muestra opción de renovación |
| Renovación | Fechas y estado cambian y el historial registra la operación |
| Edición | Cambian los datos administrativos sin alterar la plantilla biométrica |
| Eliminación: cancelar primera advertencia | No se elimina nada |
| Eliminación: texto distinto de ELIMINAR | Botón definitivo deshabilitado |
| Eliminación definitiva con doble confirmación | Desaparecen ficha, plantilla y registros dependientes en SQLite |
| Dashboard | Conteos concuerdan con consultas reales y no muestran valores simulados |
| Pantalla de check-in | Sin scroll innecesario a resolución objetivo; modal accesible |
| Reiniciar aplicación | Persisten clientes, membresías y asistencias |
| Desconectar/reconectar USB | Mensaje de error claro y recuperación controlada |

## Advertencias y limitaciones

- `SGFPM_Init: 6` o errores de lectura deben investigarse en el equipo con el lector.
- El borrado mediante SQL elimina las filas lógicas; **no certifica eliminación forense** de páginas libres, WAL, respaldos o copias del sistema. Para un borrado más fuerte, establecer política de retención, respaldo cifrado y mantenimiento SQLite (por ejemplo `secure_delete` y `VACUUM` tras evaluar sus efectos).
- El comando de borrado no implementa autenticación de administrador. Antes de producción se requiere control de permisos y registro de operaciones, sin almacenar datos biométricos en el registro.
- Validar el empaquetado y las DLL de SecuGen en el Windows real del cliente. No asumir compatibilidad con x86.

## Estado de pruebas

Los cambios en GitHub **no equivalen a compilación validada**. Marcar cada caso únicamente después de ejecutarlo en Windows con el lector físico.
