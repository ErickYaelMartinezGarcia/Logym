# VIRTUS GYM

Panel local para controlar miembros, acceso biométrico y registro de asistencia de VIRTUS GYM. La interfaz está construida con Vue 3 + Vite y la persistencia local se realiza con SQLite desde Tauri.

## Desarrollo

```bash
npm install
npm run dev
```

## Base de datos local

Al ejecutar la aplicación Tauri, SQLite guarda `gym.db` en la carpeta local de
datos de la aplicación del usuario, no junto al ejecutable. Tauri crea esa carpeta en la
primera ejecución y reutiliza el archivo existente en ejecuciones posteriores.

Si una versión anterior dejó un `gym.db` en la carpeta de trabajo, la aplicación
lo copia una sola vez a la nueva ubicación y conserva el archivo original como
respaldo. Nunca sobrescribe una base ya existente en la carpeta de datos.

La aplicación es local: cada computadora tiene su propia base de datos. Para
evitar pérdida de información, se debe respaldar `gym.db` antes de reinstalar o
trasladar la aplicación.

Para probar cámara en el navegador abre el origen que imprime Vite, normalmente `http://localhost:5173`. `getUserMedia()` solo funciona en un contexto seguro: `localhost` cuenta como origen seguro, pero abrir `dist/index.html` directamente con `file://` no. También hay que aceptar el permiso de cámara en el navegador y verificar que ninguna otra aplicación la esté usando.

## Cámara web

La pantalla **Nuevo miembro** no solicita la cámara automáticamente. Esto es intencional: el navegador y Tauri pueden bloquear solicitudes silenciosas o mostrarlas en un momento inesperado. El flujo actual es:

1. Pulsar **Activar cámara**.
2. `navigator.mediaDevices.getUserMedia({ video, audio: false })` solicita permiso y conecta el `MediaStream` al elemento `<video>`.
3. Pulsar **Tomar foto** para dibujar el frame actual en un `<canvas>` y convertirlo a JPEG base64.
4. Al salir de la pantalla se detienen todas las pistas (`track.stop()`), evitando que la cámara quede encendida.

Si no se abre en una laptop con cámara, el mensaje diferencia entre: API no disponible (origen inseguro o WebView sin soporte), permiso rechazado, dispositivo inexistente y otros errores. En Chrome/Edge se puede revisar el candado de la URL y **Configuración del sitio > Cámara**. En la aplicación Tauri debe probarse con `npm run tauri dev`, no abriendo un HTML compilado manualmente.

Para producción conviene reemplazar el base64 por un archivo administrado por la aplicación (por ejemplo, guardarlo en `appData`) y persistir únicamente la ruta. El base64 actual se mantiene como una implementación local sencilla.

## Lector de huellas

`src-tauri/src/main.rs` contiene hoy un punto de integración controlado:

- `capturar_huella` devuelve `template_fake_dp4500_*`; no lee hardware real.
- `guardar_template` mantiene temporalmente el template en memoria.
- `comparar_huella` compara texto y sirve únicamente para pruebas.

Para conectar un lector real (por ejemplo DigitalPersona 4500):

1. Instalar el SDK oficial y su driver en el equipo donde correrá Tauri.
2. Crear un módulo Rust que use el SDK o un pequeño servicio local del fabricante; evitar cargar DLLs directamente desde Vue.
3. Sustituir el cuerpo de `capturar_huella` por la secuencia del SDK: inicializar lector, esperar evento de dedo, adquirir imagen, crear template y devolver un identificador/template serializable.
4. Guardar el template cifrado o protegido en el almacenamiento local; no dejarlo en `PLANTILLAS`, porque ese mapa se pierde al cerrar la aplicación.
5. En el registro, capturar dos muestras y validar la calidad/coincidencia con el SDK antes de guardar.
6. En check-in, capturar una muestra y ejecutar la búsqueda 1:N contra los templates almacenados; registrar la asistencia solo si el SDK devuelve una coincidencia válida.
7. Implementar errores de lector desconectado, timeout, dedo retirado, mala calidad y cancelación para que la UI pueda mostrar estados específicos.

No es posible activar un lector de huellas desde `npm run dev`: el navegador no expone acceso genérico a lectores biométricos USB. El comando Tauri es el puente correcto para hardware, pero requiere el SDK y el driver del modelo elegido.

## Estructura visual

- `src/components/AppShell.vue`: menú hamburguesa y sidebar desplegable compartido.
- `src/views/Dashboard.vue`: Dashboard de métricas y miembros.
- `src/views/Registro.vue`: alta, cámara y captura biométrica.
- `src/views/CheckIn.vue`: acceso de miembros.
- `src/style.css`: identidad VIRTUS GYM, fondo oscuro y acentos rojo neón.


Voy a revisar la configuración de datos, las vistas y la parte de Tauri para reconstruir el flujo real del proyecto y señalar qué está implementado hoy frente a lo que todavía falta.

Read 
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src\\db.js"
}

Read 
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\package.json"
}

Read 
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src\\views\\Registro.vue"
}

Read 
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src\\views\\CheckIn.vue"
}

Read 
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src-tauri\\src\\main.rs"
}

Read 
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src-tauri\\Cargo.toml"
}

Read 
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\README.md"
}

La arquitectura actual sí tiene un “backend” local: Vue llama comandos de Rust mediante Tauri y Rust usa SQLite. Ahora voy a ubicar las implementaciones de esos comandos y los puntos exactos donde se invocan desde las pantallas, porque ahí se define realmente cómo se insertan, consultan y eliminan los datos.

Search for `^(#\[tauri::command\]|fn |async fn |struct |static |const )`

Search for `inicializarBaseDeDatos|guardarClienteEnBaseDeDatos|listarClientes|obtenerMetr...`

Read , lines 1 to 220
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src-tauri\\src\\main.rs",
  "view_range": [
    1,
    220
  ]
}

Read , lines 1 to 180
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src\\views\\Registro.vue",
  "view_range": [
    1,
    180
  ]
}

Read , lines 1 to 180
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src\\views\\CheckIn.vue",
  "view_range": [
    1,
    180
  ]
}

Read , lines 1 to 180
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src\\views\\Dashboard.vue",
  "view_range": [
    1,
    180
  ]
}

Read , lines 180 to 360
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src-tauri\\src\\main.rs",
  "view_range": [
    180,
    360
  ]
}

Read , lines 360 to 520
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src-tauri\\src\\main.rs",
  "view_range": [
    360,
    520
  ]
}

Read 
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src\\router.js"
}

Read 
Completed with input: {
  "path": "C:\\Users\\EricY\\OneDrive\\Desktop\\Logym\\Logym\\src\\App.vue"
}

Search for `asistencias|membresias_historial|INSERT|DELETE|UPDATE|AUTOINCREMENT|FOREIGN KEY`

## Resumen de la arquitectura actual

Este proyecto no tiene un backend web tradicional como Node, Express o Laravel. Es una aplicación de escritorio local:

```text
Vue 3
  ↓ invoke(...)
Tauri
  ↓ comandos Rust
Rust + rusqlite
  ↓ SQL
SQLite: gym.db
```

La interfaz está en Vue y el “backend” está principalmente en `main.rs`. La comunicación entre ambos lados está centralizada parcialmente en `db.js`.

---

# 1. Flujo general de trabajo

## A. Arranque de la aplicación

Cuando se abre Dashboard o Check-in:

1. Vue ejecuta:

```js
await inicializarBaseDeDatos()
```

2. Esta función está en `db.js`:

```js
export async function inicializarBaseDeDatos() {
  return await invoke('inicializar_db')
}
```

3. `invoke('inicializar_db')` llama al comando Rust:

```rust
#[tauri::command]
fn inicializar_db() -> Result<bool, String>
```

4. Rust abre o crea:

```rust
Connection::open("gym.db")
```

5. Después crea estas tablas si no existen:

- `clientes`
- `asistencias`
- `membresias_historial`

Esto está en `inicializar_db`.

---

# 2. ¿Dónde está la base de datos?

Actualmente se utiliza este código:

```rust
Connection::open("gym.db")
```

Por lo tanto, SQLite crea `gym.db` en el directorio de trabajo desde el que se ejecuta Tauri.

Durante desarrollo puede quedar en una ubicación relativa al proyecto o al proceso que inicia la aplicación. En producción esto no es lo más recomendable, porque la ubicación puede variar.

## Recomendación

La base de datos debería guardarse en el directorio oficial de datos de la aplicación, por ejemplo:

```text
AppData/Local/Logym/gym.db
```

o mediante la ruta de datos de aplicación que proporciona Tauri.

La idea sería que el backend haga algo equivalente a:

```text
app_data_dir/
└── gym.db
```

Así la base de datos no depende de desde dónde se abrió el ejecutable.

---

# 3. Estructura actual de la base de datos

## Tabla `clientes`

Se crea así:

```sql
CREATE TABLE clientes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    nombre TEXT NOT NULL,
    telefono TEXT,
    foto_base64 TEXT,
    template_huella TEXT,
    fecha_registro TEXT,
    fecha_inicio_membresia TEXT,
    fecha_fin_membresia TEXT
);
```

Contiene la información principal del cliente:

| Campo | Uso |
|---|---|
| `id` | Identificador único |
| `nombre` | Nombre del cliente |
| `telefono` | Teléfono |
| `foto_base64` | Foto actualmente guardada como texto base64 |
| `template_huella` | Plantilla biométrica |
| `fecha_registro` | Fecha de alta |
| `fecha_inicio_membresia` | Inicio de membresía |
| `fecha_fin_membresia` | Fin de membresía |

## Tabla `asistencias`

```sql
CREATE TABLE asistencias (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cliente_id INTEGER NOT NULL,
    fecha_hora TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id)
);
```

Esta tabla está creada, pero actualmente falta implementar la operación que inserta una asistencia cuando el cliente hace check-in.

Actualmente solamente se consulta para obtener el número de asistencias del día:

```sql
SELECT COUNT(*)
FROM asistencias
WHERE date(fecha_hora) = date('now')
```

Eso ocurre en `obtener_metricas`.

## Tabla `membresias_historial`

Guarda las altas y renovaciones:

```sql
CREATE TABLE membresias_historial (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cliente_id INTEGER NOT NULL,
    fecha_inicio TEXT NOT NULL,
    fecha_fin TEXT NOT NULL,
    fecha_pago TEXT NOT NULL,
    monto REAL DEFAULT 0,
    tipo TEXT,
    observaciones TEXT
);
```

Esto permite conservar el historial de pagos y membresías, aunque actualmente no existe una pantalla para consultarlo.

---

# 4. Flujo para registrar un cliente

La pantalla está en `Registro.vue`.

## Paso 1: Capturar datos

El usuario introduce:

- Nombre
- Teléfono
- Fecha de registro
- Inicio de membresía
- Fin de membresía
- Foto
- Dos capturas de huella

La foto se obtiene de la cámara:

```js
navigator.mediaDevices.getUserMedia(...)
```

Después se convierte a base64:

```js
canvas.toDataURL('image/jpeg', .86)
```

Actualmente la foto se guarda directamente en la columna `foto_base64`.

## Paso 2: Capturar huella

La pantalla ejecuta:

```js
const template = await invoke('capturar_huella')
```

Esto llama a:

```rust
#[tauri::command]
fn capturar_huella() -> Result<String, String>
```

Pero actualmente la huella es simulada. El código busca la última huella guardada y la devuelve, o genera un texto como:

```text
template_fake_dp4500_...
```

Por eso todavía no está conectado realmente el lector DigitalPersona.

## Paso 3: Guardar cliente

La pantalla llama:

```js
guardarClienteEnBaseDeDatos(datosCliente)
```

En `db.js`, esto ejecuta:

```js
invoke('guardar_cliente', { datos: datosCliente })
```

Rust recibe los datos en:

```rust
fn guardar_cliente(datos: DatosCliente) -> Result<i64, String>
```

Después ejecuta:

```sql
INSERT INTO clientes (...)
VALUES (...)
```

Luego obtiene el ID generado:

```rust
let cliente_id = conexion.last_insert_rowid();
```

Finalmente guarda el primer registro en `membresias_historial` y devuelve el ID al frontend.

Flujo completo:

```text
Registro.vue
  ↓ guardarCliente()
db.js
  ↓ guardarClienteEnBaseDeDatos()
Tauri
  ↓ invoke('guardar_cliente')
main.rs
  ↓ guardar_cliente()
SQLite
  ↓ INSERT INTO clientes
  ↓ INSERT INTO membresias_historial
  ↓ devuelve cliente_id
Vue
```

---

# 5. Flujo para consultar clientes

## Dashboard

La pantalla está en `Dashboard.vue`.

Al cargar:

```js
await inicializarBaseDeDatos()
clientes.value = await listarClientes()
clientesMembresia.value = await listarClientesMembresia()
const datos = await obtenerMetricasDashboard()
```

## Consulta simple de clientes

La función frontend:

```js
listarClientes()
```

llama a:

```rust
fn listar_clientes()
```

La consulta SQL actual es:

```sql
SELECT id, nombre, telefono, fecha_registro
FROM clientes
ORDER BY id DESC
LIMIT 20
```

## Consulta de membresías

La función:

```js
listarClientesMembresia()
```

llama a:

```rust
fn listar_clientes_membresia()
```

Esta consulta calcula también el estado:

```text
vencida
por_vencer
activa
sin_fecha
```

La fecha se compara con la fecha actual usando SQLite:

```sql
julianday(fecha_fin_membresia) - julianday(date('now'))
```

La búsqueda por nombre, teléfono o ID se hace actualmente en Vue, no en SQLite:

```js
clientesMembresia.value.filter(...)
```

Para pocos clientes está bien. Si la base crece mucho, sería mejor hacer la búsqueda directamente en SQL.

---

# 6. Flujo del check-in

La pantalla está en `CheckIn.vue`.

## Paso 1: Capturar huella

```js
const template = await invoke('capturar_huella')
```

## Paso 2: Buscar cliente

```js
const cliente = await buscarClientePorHuella(template)
```

En `db.js`, esto llama:

```js
invoke('buscar_cliente_por_huella', { template })
```

Rust ejecuta:

```sql
SELECT ...
FROM clientes
WHERE template_huella = ?1
ORDER BY id DESC
LIMIT 1
```

Esto está dentro de `obtener_cliente_checkin_por_template`.

## Paso 3: Mostrar el resultado

Si encuentra coincidencia, Vue muestra:

- Nombre
- ID
- Teléfono
- Foto
- Estado de membresía
- Fecha de inicio
- Fecha de fin

## Importante: actualmente no se registra la asistencia

El flujo actual busca al cliente, pero no ejecuta ningún:

```sql
INSERT INTO asistencias (...)
```

Por eso la métrica “Entradas de hoy” probablemente se mantiene en cero.

El flujo completo debería terminar así:

```text
Capturar huella
  ↓
Buscar cliente
  ↓
Validar membresía
  ↓
INSERT INTO asistencias (cliente_id)
  ↓
Mostrar acceso permitido
```

---

# 7. Flujo de renovación

Cuando se renueva desde Check-in:

```js
renovarMembresiaCliente(clienteId, dias, monto)
```

llama a:

```rust
fn renovar_membresia(datos: DatosRenovacion)
```

Actualmente realiza:

```sql
UPDATE clientes
SET fecha_registro = ?1,
    fecha_inicio_membresia = ?2,
    fecha_fin_membresia = ?3
WHERE id = ?4
```

Luego inserta el pago en:

```sql
membresias_historial
```

## Hay un detalle importante

La renovación está cambiando también `fecha_registro`:

```sql
SET fecha_registro = ?1
```

Eso no debería ocurrir. La fecha de registro debe representar cuándo se creó el cliente y no cambiar cada vez que paga.

La renovación debería actualizar solamente:

```sql
fecha_inicio_membresia
fecha_fin_membresia
```

Y dejar intacta `fecha_registro`.

Además, normalmente una renovación de un cliente todavía activo debería comenzar después de la fecha actual de vencimiento, no necesariamente desde hoy. Por ejemplo:

```text
Membresía termina: 30 de noviembre
Renueva 30 días el 15 de noviembre

Resultado correcto:
Nuevo inicio: 1 de diciembre
Nuevo fin: 30 de diciembre
```

Eso evita perder días ya pagados.

---

# 8. ¿Cómo se asignan los IDs?

Actualmente se usa:

```sql
id INTEGER PRIMARY KEY AUTOINCREMENT
```

Y se obtiene así:

```rust
let cliente_id = conexion.last_insert_rowid();
```

## ¿Es una buena manera?

Sí, para este caso es una forma correcta y común.

Características:

- SQLite genera el ID automáticamente.
- No se debe generar el ID desde Vue.
- No se debe usar el nombre o teléfono como ID.
- El ID sirve como clave interna estable.
- Las tablas relacionadas usan `cliente_id`.

Ejemplo:

```text
Cliente:
id = 15
nombre = "Carlos Pérez"

Asistencia:
cliente_id = 15
```

## ¿Qué pasa si elimino al cliente 15?

El ID 15 no debería reutilizarse. Si después registras otro cliente, normalmente recibirá otro número:

```text
Cliente eliminado: 15
Nuevo cliente: 16
```

Esto es bueno porque evita confundir registros históricos.

Los huecos son normales:

```text
10, 11, 12, 14, 15
```

El ID no tiene que ser una numeración visual continua.

---

# 9. ¿Cómo eliminar un cliente?

Actualmente no existe una función de eliminación.

No hay:

```rust
fn eliminar_cliente(...)
```

ni tampoco:

```js
eliminarCliente(...)
```

Por lo tanto, la interfaz no puede eliminar clientes todavía.

## Opción recomendada: baja lógica

Para un gimnasio, generalmente es mejor no borrar físicamente al cliente. Se puede agregar:

```sql
activo INTEGER NOT NULL DEFAULT 1
```

o:

```sql
eliminado_en TEXT
```

Entonces, al “eliminar”:

```sql
UPDATE clientes
SET activo = 0
WHERE id = ?1
```

Las consultas normales filtran:

```sql
WHERE activo = 1
```

Ventajas:

- Conservas el historial de membresías.
- Conservas las asistencias.
- Puedes recuperar al cliente.
- No rompes relaciones.
- No pierdes información contable o administrativa.
- El ID no cambia.

El flujo sería:

```text
Administrador pulsa "Eliminar"
  ↓
Se pide confirmación
  ↓
Rust recibe cliente_id
  ↓
UPDATE clientes SET activo = 0
  ↓
El cliente desaparece de las listas activas
  ↓
Sus asistencias e historial permanecen
```

## Opción de borrado físico

También se podría hacer:

```sql
DELETE FROM clientes
WHERE id = ?1
```

Pero esto es más peligroso porque:

- Puede dejar asistencias huérfanas.
- Puede dejar historial sin cliente.
- Se pierde información.
- No hay recuperación.
- Las huellas y fotos desaparecen definitivamente.

Si se desea borrar físicamente, primero hay que definir la política:

```sql
PRAGMA foreign_keys = ON;
```

y decidir si las relaciones usarán:

```sql
ON DELETE CASCADE
```

Por ejemplo:

```sql
FOREIGN KEY (cliente_id)
REFERENCES clientes(id)
ON DELETE CASCADE
```

Eso eliminaría también asistencias e historial. Para datos de clientes reales, normalmente recomendaría la baja lógica.

---

# 10. Capacidades actuales y faltantes

## Actualmente sí existe

- Crear la base local.
- Crear clientes.
- Guardar fotos.
- Guardar membresías.
- Guardar historial inicial y renovaciones.
- Consultar clientes.
- Consultar estado de membresía.
- Buscar por huella simulada.
- Renovar membresías.
- Calcular métricas básicas.
- Mostrar vencimientos.

## Actualmente falta o está incompleto

- Integrar el lector de huellas real.
- Registrar asistencias en cada check-in.
- Mostrar historial de pagos.
- Editar clientes.
- Dar de baja clientes.
- Reactivar clientes.
- Usar una ruta estable para `gym.db`.
- Validar que la segunda huella coincida con la primera.
- Proteger o cifrar los templates biométricos.
- Evitar duplicados de huella.
- Usar transacciones para operaciones relacionadas.
- Corregir que la renovación cambie `fecha_registro`.
- Definir permisos o usuarios administradores.

---

# 11. Flujo recomendado completo

```text
INICIO
  ↓
inicializar_db()
  ↓
Abrir gym.db
  ↓
Crear/migrar tablas
  ↓

REGISTRAR CLIENTE
  ↓
Capturar datos personales
  ↓
Capturar foto
  ↓
Capturar huella 1
  ↓
Capturar huella 2
  ↓
Validar que ambas coincidan
  ↓
INSERT clientes
  ↓
Obtener last_insert_rowid()
  ↓
INSERT membresias_historial
  ↓
Mostrar cliente registrado


CHECK-IN
  ↓
Capturar huella real
  ↓
Buscar template_huella
  ↓
Obtener cliente
  ↓
Calcular estado de membresía
  ↓
Si tiene acceso permitido:
    INSERT asistencias
  ↓
Mostrar acceso permitido


RENOVACIÓN
  ↓
Buscar cliente por ID
  ↓
Calcular nuevo periodo
  ↓
UPDATE clientes
  ↓
INSERT membresias_historial
  ↓
Mostrar nueva fecha


BAJA
  ↓
Confirmar acción
  ↓
UPDATE clientes SET activo = 0
  ↓
Excluirlo de consultas normales
  ↓
Conservar historial y asistencias
```

En resumen: la base y las operaciones principales ya están iniciadas, pero todavía faltan tres piezas importantes para tener un backend sólido: una ruta estable para SQLite, el registro real de asistencias y la baja lógica de clientes.