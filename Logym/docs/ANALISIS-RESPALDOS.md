# Análisis de respaldos SQLite (sin implementación)

## Estado actual

- La base de datos `gym.db` se almacena en el directorio de datos de la aplicación de Tauri.
- Cada conexión activa `PRAGMA journal_mode=WAL` y `PRAGMA foreign_keys=ON`.
- La aplicación no dispone de copias de seguridad programadas, restauración, verificación periódica de integridad ni retención.
- La migración de una base heredada utiliza `fs::copy` y no comprueba el resultado; no constituye un mecanismo de respaldo.
- El expediente incluye información personal, fotografías y plantillas biométricas: un respaldo sin cifrar puede exponer datos sensibles.

## Riesgos

1. Copiar únicamente `gym.db` mientras la aplicación escribe en WAL puede producir una copia inconsistente o incompleta; pueden existir `gym.db-wal` y `gym.db-shm`.
2. Un fallo de disco, borrado accidental o corrupción puede ocasionar pérdida de clientes, huellas, asistencias e historial.
3. Guardar una copia sin cifrar en una carpeta compartida, USB o nube expone información biométrica.
4. La eliminación de clientes en la base activa no borra automáticamente sus datos de respaldos históricos.

## Diseño propuesto para una fase futura

- Utilizar la API oficial de backup online de SQLite o `VACUUM INTO` desde una conexión consistente, no `fs::copy` sobre la base activa.
- Crear primero un archivo temporal, verificar `PRAGMA integrity_check` y promoverlo de forma atómica.
- Cifrar el archivo de respaldo, con clave fuera del propio respaldo y procedimiento seguro de recuperación.
- Política configurable de retención y eliminación, incluyendo el tratamiento de datos biométricos borrados.
- Restringir la ubicación y permisos del respaldo, evitar incluirlo en logs o telemetría.
- Probar restauración en una base aislada antes de reemplazar la activa y documentar el procedimiento de recuperación.
- Determinar periodicidad, retención y responsable operativo según requisitos del gimnasio y normativa aplicable.

## Alcance de este cambio

**Solo análisis.** No se crean tareas, archivos de respaldo, exportaciones, rutas nuevas ni comandos de restauración.
