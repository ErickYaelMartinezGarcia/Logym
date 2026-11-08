# Arquitectura del frontend

El frontend está organizado por responsabilidad. Las vistas componen la pantalla, los composables contienen estado y casos de uso, y los servicios son la única frontera con Tauri.

## Flujo de datos

```text
views/*.vue
    ↓ usa
composables/*.js
    ↓ llama
services/tauriApi.js
    ↓ invoke
Comandos Tauri/Rust
```

## Capas

### `views`

Contienen la estructura de cada pantalla y conectan eventos del template con un composable.

- `Dashboard.vue`: presentación del dashboard.
- `CheckIn.vue`: presentación del acceso y renovación.
- `Registro.vue`: presentación del alta de cliente.

Una vista no debe importar `invoke` ni conocer los nombres de comandos Tauri.

### `composables`

Contienen estado reactivo, validaciones y coordinación de una pantalla o flujo:

- `useDashboard.js`: carga y filtra datos del dashboard.
- `useCheckIn.js`: captura, búsqueda, validación de asistencia y renovación.
- `useRegistroCliente.js`: validación y guardado de un cliente.
- `useCamera.js`: ciclo de vida de cámara y captura de foto.

Esta capa es el lugar para lógica reutilizable relacionada con la interfaz.

### `services`

`services/tauriApi.js` es la frontera de infraestructura. Centraliza `invoke`, los argumentos enviados al backend y el manejo de errores de comunicación.

Para agregar una operación nueva:

1. Agregar una función en `tauriApi.js`.
2. Consumirla desde un composable.
3. Exponer el estado o acción a la vista.

No se debe llamar `invoke` directamente desde una vista o componente.

### `components`

Contienen piezas visuales reutilizables. `AppShell.vue` define la navegación y el marco común de las pantallas.

Un componente visual no debería cargar datos de SQLite ni conocer comandos Tauri.

### `style.css`

Tailwind se importa como base y utilidades. Las clases de componentes reutilizables se agrupan bajo `@layer components`; esto evita estilos inline y mantiene nombres visuales consistentes (`surface`, `data-table`, `metric-card`, etc.).

## Compatibilidad

`db.js` conserva exportaciones antiguas como fachada temporal. El código nuevo debe importar desde `services/tauriApi.js`; cuando no existan consumidores externos, `db.js` podrá eliminarse.
