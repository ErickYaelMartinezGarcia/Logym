# Backend de VIRTUS GYM

Este directorio contiene el backend Tauri/Rust de la aplicación. La interfaz Vue no cambió sus comandos: sigue comunicándose mediante `invoke` con los mismos nombres.

## Flujo general

```text
Vue / src/db.js
        │ invoke(nombre_del_comando)
        ▼
commands.rs          Adaptador Tauri y entrada pública
        ▼
services.rs          Reglas de negocio y coordinación
        ▼
repositories.rs      Consultas y escrituras SQLite
        ▼
database.rs           Ubicación, conexión y esquema de gym.db
```

La biometría está aislada en `biometria.rs` porque actualmente contiene un mock del lector. Ahí se reemplazará la simulación cuando se integre el SDK real.

## Archivos y responsabilidades

| Archivo | Responsabilidad |
| --- | --- |
| `main.rs` | Registra módulos, plugins y comandos Tauri. No contiene reglas de negocio ni SQL. |
| `commands.rs` | Expone funciones con `#[tauri::command]` y delega a servicios. |
| `models.rs` | Define los datos de entrada y salida compartidos con el frontend. |
| `services.rs` | Valida reglas, abre la base y coordina repositorios. |
| `repositories.rs` | Contiene SQL y transforma filas SQLite en modelos Rust. |
| `database.rs` | Resuelve la ruta de `gym.db`, crea el esquema e índices. |
| `biometria.rs` | Encapsula captura, almacenamiento temporal y comparación de huellas. |

## Comandos disponibles

| Comando | Uso |
| --- | --- |
| `inicializar_db` | Crea tablas e índices necesarios. |
| `guardar_cliente` | Guarda un cliente y su alta inicial en el historial de membresías. |
| `listar_clientes` | Devuelve las altas recientes para el dashboard. |
| `listar_clientes_membresia` | Devuelve clientes con estado y días restantes de membresía. |
| `buscar_cliente_por_huella` | Busca la identidad asociada a un template. |
| `registrar_asistencia` | Registra una asistencia válida; solo permite una por cliente y fecha. |
| `renovar_membresia` | Autoriza una renovación mensual de 30 días y agrega el movimiento al historial; no procesa cobros. |
| `obtener_metricas` | Devuelve entradas de hoy, últimos 7 días, día y hora de mayor afluencia. |
| `capturar_huella` | Mock temporal del lector DigitalPersona 4500. |
| `guardar_template` | Guarda temporalmente un template en memoria. |
| `comparar_huella` | Compara templates en el mock actual. |

## Base de datos

La base se llama `gym.db` y se resuelve en la carpeta de datos local de Tauri. Si existe un `gym.db` legado en el directorio de trabajo, `database.rs` lo copia sin eliminar el original.

### Tablas

```text
clientes
  id
  nombre
  telefono
  telefono_emergencia
  foto_base64
  template_huella
  fecha_registro
  fecha_inicio_membresia
  fecha_fin_membresia

asistencias
  id
  cliente_id → clientes.id
  fecha_hora

membresias_historial
  id
  cliente_id → clientes.id
  fecha_inicio
  fecha_fin
  fecha_pago
  tipo
  observaciones
  creado_en
```

`idx_asistencias_cliente_fecha` acelera la validación de asistencia diaria.

El historial registra altas y renovaciones administrativas. Actualmente no existe un flujo de caja ni se procesa ningún pago.

## Regla de asistencia

El check-in sigue este flujo:

```text
capturar_huella
    ↓
buscar_cliente_por_huella
    ↓
validar membresía
    ↓
registrar_asistencia
    ↓
INSERT solo si no existe otra asistencia del mismo cliente en date('now')
```

La validación de duplicados está en el backend, no solo en Vue. El `INSERT ... SELECT ... WHERE NOT EXISTS` evita registrar dos asistencias ante reintentos rápidos.

## Cómo agregar una función nueva

1. Agregar o actualizar el modelo en `models.rs`.
2. Implementar la consulta en `repositories.rs`.
3. Implementar validaciones y coordinación en `services.rs`.
4. Exponer un wrapper Tauri en `commands.rs`.
5. Registrar el comando en `main.rs`.
6. Agregar el wrapper de `invoke` en `src/db.js`.

No se debe colocar SQL directamente en `commands.rs` ni lógica de negocio en `main.rs`.
