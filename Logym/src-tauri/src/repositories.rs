use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::models::{
    ClienteCheckIn, ClienteMembresia, ClienteTabla, DatosCliente, DatosRenovacion, Metricas,
};

pub fn guardar_cliente(conexion: &Connection, datos: DatosCliente) -> Result<i64, String> {
    conexion
        .execute(
            "INSERT INTO clientes (nombre, telefono, telefono_emergencia, fecha_registro, fecha_inicio_membresia, fecha_fin_membresia, foto_base64, template_huella)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                datos.nombre,
                datos.telefono,
                datos.telefono_emergencia,
                datos.fecha_registro,
                datos.fecha_inicio_membresia,
                datos.fecha_fin_membresia,
                datos.foto_base64,
                datos.template_huella
            ],
        )
        .map_err(|e| e.to_string())?;

    let cliente_id = conexion.last_insert_rowid();
    conexion
        .execute(
            "INSERT INTO membresias_historial (cliente_id, fecha_inicio, fecha_fin, fecha_pago, tipo, observaciones)
             VALUES (?1, ?2, ?3, ?4, 'alta', 'Alta inicial del cliente')",
            params![
                cliente_id,
                datos.fecha_inicio_membresia,
                datos.fecha_fin_membresia,
                datos.fecha_registro
            ],
        )
        .map_err(|e| e.to_string())?;

    Ok(cliente_id)
}

fn calcular_estado_membresia(fecha_fin: &str) -> (String, i64) {
    if fecha_fin.trim().is_empty() {
        return (String::from("sin_fecha"), 9999);
    }

    let hoy = Utc::now().date_naive();
    match chrono::NaiveDate::parse_from_str(fecha_fin, "%Y-%m-%d") {
        Ok(fin) => {
            let dias = (fin - hoy).num_days();
            if dias < 0 {
                (String::from("vencida"), dias)
            } else if dias <= 7 {
                (String::from("por_vencer"), dias)
            } else {
                (String::from("activa"), dias)
            }
        }
        Err(_) => (String::from("sin_fecha"), 9999),
    }
}

pub fn buscar_cliente_por_huella(
    conexion: &Connection,
    template: &str,
) -> Result<Option<ClienteCheckIn>, String> {
    let mut sentencia = conexion
        .prepare(
            "SELECT id, nombre, COALESCE(telefono, ''), COALESCE(telefono_emergencia, ''),
                    COALESCE(foto_base64, ''), COALESCE(fecha_registro, ''),
                    COALESCE(fecha_inicio_membresia, ''), COALESCE(fecha_fin_membresia, '')
             FROM clientes
             WHERE template_huella = ?1
             ORDER BY id DESC
             LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let resultado = sentencia.query_row(params![template], |fila| {
        Ok(ClienteCheckIn {
            id: fila.get(0)?,
            nombre: fila.get(1)?,
            telefono: fila.get(2)?,
            telefono_emergencia: fila.get(3)?,
            foto_base64: fila.get(4)?,
            fecha_registro: fila.get(5)?,
            fecha_inicio_membresia: fila.get(6)?,
            fecha_fin_membresia: fila.get(7)?,
            estado_membresia: String::new(),
            dias_restantes: 0,
        })
    });

    match resultado {
        Ok(mut cliente) => {
            let (estado, dias) = calcular_estado_membresia(&cliente.fecha_fin_membresia);
            cliente.estado_membresia = estado;
            cliente.dias_restantes = dias;
            Ok(Some(cliente))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn registrar_asistencia(conexion: &Connection, cliente_id: i64) -> Result<bool, String> {
    let fecha_fin: String = conexion
        .query_row(
            "SELECT COALESCE(fecha_fin_membresia, '') FROM clientes WHERE id = ?1",
            params![cliente_id],
            |fila| fila.get(0),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => String::from("No se encontró el cliente"),
            error => error.to_string(),
        })?;

    if fecha_fin.trim().is_empty() {
        return Err(String::from("El cliente no tiene una membresía válida"));
    }

    let membresia_vigente: i64 = conexion
        .query_row(
            "SELECT CASE WHEN date(?1) >= date('now') THEN 1 ELSE 0 END",
            params![fecha_fin],
            |fila| fila.get(0),
        )
        .map_err(|e| e.to_string())?;

    if membresia_vigente == 0 {
        return Err(String::from("La membresía del cliente está vencida"));
    }

    let filas_insertadas = conexion
        .execute(
            "INSERT INTO asistencias (cliente_id, fecha_hora)
             SELECT ?1, CURRENT_TIMESTAMP
             WHERE NOT EXISTS (
                 SELECT 1 FROM asistencias
                 WHERE cliente_id = ?1 AND date(fecha_hora) = date('now')
             )",
            params![cliente_id],
        )
        .map_err(|e| e.to_string())?;

    Ok(filas_insertadas > 0)
}

pub fn renovar_membresia(
    conexion: &Connection,
    datos: DatosRenovacion,
) -> Result<ClienteCheckIn, String> {
    let hoy = Utc::now().date_naive();
    let fecha_inicio = hoy.format("%Y-%m-%d").to_string();
    let fecha_fin = (hoy + chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    conexion
        .execute(
            "UPDATE clientes
             SET fecha_inicio_membresia = ?1, fecha_fin_membresia = ?2
             WHERE id = ?3",
            params![fecha_inicio, fecha_fin, datos.cliente_id],
        )
        .map_err(|e| e.to_string())?;

    if conexion.changes() == 0 {
        return Err(String::from("No se encontró el cliente para renovar"));
    }

    conexion
        .execute(
            "INSERT INTO membresias_historial
             (cliente_id, fecha_inicio, fecha_fin, fecha_pago, tipo, observaciones)
             VALUES (?1, ?2, ?3, ?4, 'renovacion',
                     'Renovación autorizada desde panel de check-in')",
            params![datos.cliente_id, fecha_inicio, fecha_fin, fecha_inicio],
        )
        .map_err(|e| e.to_string())?;

    let template: String = conexion
        .query_row(
            "SELECT template_huella FROM clientes WHERE id = ?1",
            params![datos.cliente_id],
            |fila| fila.get(0),
        )
        .map_err(|e| e.to_string())?;

    buscar_cliente_por_huella(conexion, &template)?
        .ok_or_else(|| String::from("No fue posible cargar el cliente renovado"))
}

pub fn listar_clientes_membresia(conexion: &Connection) -> Result<Vec<ClienteMembresia>, String> {
    let mut sentencia = conexion
        .prepare(
            "SELECT id, nombre, COALESCE(telefono, ''), COALESCE(telefono_emergencia, ''), COALESCE(fecha_registro, ''),
                    COALESCE(fecha_inicio_membresia, ''), COALESCE(fecha_fin_membresia, ''),
                    CASE
                        WHEN COALESCE(fecha_fin_membresia, '') = '' THEN 'sin_fecha'
                        WHEN julianday(fecha_fin_membresia) < julianday(date('now')) THEN 'vencida'
                        WHEN julianday(fecha_fin_membresia) <= julianday(date('now', '+7 day')) THEN 'por_vencer'
                        ELSE 'activa'
                    END,
                    CASE
                        WHEN COALESCE(fecha_fin_membresia, '') = '' THEN 9999
                        ELSE CAST(julianday(fecha_fin_membresia) - julianday(date('now')) AS INTEGER)
                    END
             FROM clientes
             ORDER BY
                CASE
                    WHEN COALESCE(fecha_fin_membresia, '') = '' THEN 3
                    WHEN julianday(fecha_fin_membresia) < julianday(date('now')) THEN 0
                    WHEN julianday(fecha_fin_membresia) <= julianday(date('now', '+7 day')) THEN 1
                    ELSE 2
                END,
                julianday(COALESCE(fecha_fin_membresia, date('now', '+3650 day'))) ASC,
                id DESC",
        )
        .map_err(|e| e.to_string())?;

    let filas = sentencia
        .query_map([], |fila| {
            Ok(ClienteMembresia {
                id: fila.get(0)?,
                nombre: fila.get(1)?,
                telefono: fila.get(2)?,
                telefono_emergencia: fila.get(3)?,
                fecha_registro: fila.get(4)?,
                fecha_inicio_membresia: fila.get(5)?,
                fecha_fin_membresia: fila.get(6)?,
                estado_membresia: fila.get(7)?,
                dias_restantes: fila.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    filas
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn listar_clientes(conexion: &Connection) -> Result<Vec<ClienteTabla>, String> {
    let mut sentencia = conexion
        .prepare(
            "SELECT id, nombre, COALESCE(telefono, ''), COALESCE(telefono_emergencia, ''), fecha_registro
             FROM clientes ORDER BY id DESC LIMIT 20",
        )
        .map_err(|e| e.to_string())?;

    let filas = sentencia
        .query_map([], |fila| {
            Ok(ClienteTabla {
                id: fila.get(0)?,
                nombre: fila.get(1)?,
                telefono: fila.get(2)?,
                telefono_emergencia: fila.get(3)?,
                fecha_registro: fila.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    filas
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

pub fn obtener_metricas(conexion: &Connection) -> Result<Metricas, String> {
    let contar = |consulta: &str| {
        conexion
            .query_row(consulta, [], |fila| fila.get::<usize, i64>(0))
            .map_err(|e| e.to_string())
    };

    let dia_numero: Option<String> = conexion
        .query_row(
            "SELECT strftime('%w', fecha_hora) FROM asistencias
             WHERE date(fecha_hora) BETWEEN date('now', '-6 day') AND date('now')
             GROUP BY strftime('%w', fecha_hora)
             ORDER BY COUNT(*) DESC, strftime('%w', fecha_hora) ASC LIMIT 1",
            [],
            |fila| fila.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let dia_mas_concurrido = match dia_numero.as_deref() {
        Some("0") => "Domingo",
        Some("1") => "Lunes",
        Some("2") => "Martes",
        Some("3") => "Miércoles",
        Some("4") => "Jueves",
        Some("5") => "Viernes",
        Some("6") => "Sábado",
        _ => "Sin datos",
    }
    .to_string();

    let hora_mayor_afluencia = conexion
        .query_row(
            "SELECT strftime('%H:00', fecha_hora) FROM asistencias
             WHERE date(fecha_hora) BETWEEN date('now', '-6 day') AND date('now')
             GROUP BY strftime('%H', fecha_hora)
             ORDER BY COUNT(*) DESC, strftime('%H', fecha_hora) ASC LIMIT 1",
            [],
            |fila| fila.get::<usize, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| String::from("Sin datos"));

    let total_clientes = contar("SELECT COUNT(*) FROM clientes")?;
    let asistencias_hoy =
        contar("SELECT COUNT(*) FROM asistencias WHERE date(fecha_hora) = date('now')")?;
    let asistencias_ultimos_7_dias = contar(
        "SELECT COUNT(*) FROM asistencias
         WHERE date(fecha_hora) BETWEEN date('now', '-6 day') AND date('now')",
    )?;
    let nuevos_semana = contar(
        "SELECT COUNT(*) FROM clientes WHERE date(fecha_registro) >= date('now', '-7 day')",
    )?;

    Ok(Metricas {
        total_clientes,
        asistencias_hoy,
        asistencias_ultimos_7_dias,
        dia_mas_concurrido,
        hora_mayor_afluencia,
        nuevos_semana,
        huella_activas: total_clientes,
    })
}
