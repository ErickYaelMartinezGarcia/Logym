use rusqlite::{params, Connection};
use crate::models::*;

pub fn guardar_cliente(conexion: &Connection, datos: DatosCliente) -> Result<i64, String> {
    let tx = conexion.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO clientes (nombre, telefono, telefono_emergencia, fecha_registro, fecha_inicio_membresia, fecha_fin_membresia, foto_base64, template_huella)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![ datos.nombre, datos.telefono, datos.telefono_emergencia, datos.fecha_registro, datos.fecha_inicio_membresia, datos.fecha_fin_membresia, datos.foto_base64, datos.template_huella ],
    ).map_err(|e| e.to_string())?;
    let id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO membresias_historial (cliente_id, fecha_inicio, fecha_fin, fecha_pago, tipo, observaciones)
         VALUES (?1, ?2, ?3, ?4, 'alta', 'Alta inicial')",
        params![id, datos.fecha_inicio_membresia, datos.fecha_fin_membresia, datos.fecha_registro],
    ).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(id)
}

pub fn listar_clientes(conexion: &Connection) -> Result<Vec<ClienteTabla>, String> {
    let mut stmt = conexion.prepare("SELECT id, nombre, telefono, fecha_fin_membresia, COALESCE(foto_base64,'') FROM clientes ORDER BY id DESC LIMIT 100").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(ClienteTabla{ id: r.get(0)?, nombre: r.get(1)?, telefono: r.get(2)?, fecha_fin_membresia: r.get(3)?, foto_base64: r.get(4)? })).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Lista clientes activos e inactivos para su administración.
pub fn listar_clientes_membresia(conexion: &Connection) -> Result<Vec<ClienteMembresia>, String> {
    let mut stmt = conexion.prepare(
        "SELECT id, nombre, COALESCE(telefono,''), COALESCE(fecha_fin_membresia,''), CAST(julianday(date(fecha_fin_membresia)) - julianday(date('now','localtime')) AS INTEGER), activo, COALESCE(telefono_emergencia,''), COALESCE(fecha_registro,''), COALESCE(fecha_inicio_membresia,'') FROM clientes ORDER BY nombre"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| {
        let dias: Option<i64> = r.get(4)?;
        let fecha_inicio: String = r.get(8)?;
        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
        let estado = match dias {
            _ if fecha_inicio > hoy => "pendiente",
            None => "sin_fecha",
            Some(n) if n < 0 => "vencida",
            Some(n) if n <= 7 => "por_vencer",
            _ => "activa"
        }.to_string();
        Ok(ClienteMembresia {
            id: r.get(0)?, nombre: r.get(1)?, telefono: r.get(2)?,
            fecha_fin_membresia: r.get(3)?, dias_restantes: dias.unwrap_or(0),
            estado_membresia: estado, activo: r.get::<_, i64>(5)? != 0,
            telefono_emergencia: r.get(6)?, fecha_registro: r.get(7)?,
            fecha_inicio_membresia: r.get(8)?
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Registra una sola asistencia por cliente y día, únicamente si tiene acceso vigente.
/// Usa la misma fecha local para escritura y consulta del dashboard.
pub fn registrar_asistencia(conexion: &Connection, cliente_id: i64) -> Result<String, String> {
    let cliente = conexion.query_row(
        "SELECT activo, date(fecha_inicio_membresia) <= date('now','localtime') AND date(fecha_fin_membresia) >= date('now','localtime') FROM clientes WHERE id = ?1",
        params![cliente_id],
        |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?.unwrap_or(0))),
    ).map_err(|_| "Cliente no encontrado".to_string())?;
    if cliente.0 != 1 { return Err("Acceso denegado: cliente desactivado".into()); }
    if cliente.1 != 1 {
        let (inicio, fin): (String, String) = conexion.query_row(
            "SELECT COALESCE(fecha_inicio_membresia,''), COALESCE(fecha_fin_membresia,'') FROM clientes WHERE id=?1",
            params![cliente_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).map_err(|e| e.to_string())?;
        let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
        if inicio > hoy { return Err(format!("Membresía pendiente: comienza el {inicio}")); }
        if fin < hoy { return Err(format!("Membresía vencida: terminó el {fin}")); }
        return Err("Acceso denegado: fechas de membresía inválidas".into());
    }

    let n = conexion.execute(
        "INSERT INTO asistencias (cliente_id, fecha_hora)
         SELECT ?1, datetime('now','localtime')
         WHERE NOT EXISTS (
             SELECT 1 FROM asistencias
             WHERE cliente_id = ?1 AND date(fecha_hora) = date('now','localtime')
         )",
        params![cliente_id],
    ).map_err(|e| format!("No se pudo guardar la asistencia: {e}"))?;
    if n == 0 { return Err("Ya registró asistencia hoy".into()); }
    Ok("Asistencia registrada".into())
}

pub fn renovar_membresia(conexion: &Connection, cliente_id: i64) -> Result<String, String> {
    let tx = conexion.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute("UPDATE clientes SET fecha_inicio_membresia = date('now','localtime'), fecha_fin_membresia = date('now','localtime','+30 day') WHERE id = ?1", params![cliente_id]).map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO membresias_historial (cliente_id, fecha_inicio, fecha_fin, fecha_pago, tipo) VALUES (?1, date('now','localtime'), date('now','localtime','+30 day'), datetime('now','localtime'), 'renovacion')", params![cliente_id]).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok("Renovada 30 días".into())
}

/// Obtiene indicadores exclusivamente de datos persistidos; nunca valores simulados.
pub fn obtener_metricas(conexion: &Connection) -> Result<Metricas, String> {
    fn count(conn: &Connection, sql: &str) -> Result<i64, String> {
        conn.query_row(sql, [], |row| row.get(0)).map_err(|e| e.to_string())
    }
    let total_clientes = count(conexion, "SELECT COUNT(*) FROM clientes WHERE activo = 1")?;
    let asistencias_hoy = count(conexion, "SELECT COUNT(*) FROM asistencias WHERE date(fecha_hora) = date('now','localtime')")?;
    let nuevos_semana = count(conexion, "SELECT COUNT(*) FROM clientes WHERE activo = 1 AND date(fecha_registro) >= date('now','localtime','-6 days')")?;
    let huella_activas = count(conexion, "SELECT COUNT(*) FROM clientes WHERE activo = 1 AND COALESCE(template_huella,'') <> ''")?;
    let asistencias_ultimos_7_dias = count(conexion, "SELECT COUNT(*) FROM asistencias WHERE date(fecha_hora) BETWEEN date('now','localtime','-6 days') AND date('now','localtime')")?;
    let dia_mas_concurrido: String = conexion.query_row(
        "SELECT date(fecha_hora) FROM asistencias WHERE date(fecha_hora) BETWEEN date('now','localtime','-6 days') AND date('now','localtime') GROUP BY date(fecha_hora) ORDER BY COUNT(*) DESC, date(fecha_hora) DESC LIMIT 1",
        [], |row| row.get(0)
    ).unwrap_or_else(|_| "Sin datos".to_string());
    let hora_mayor_afluencia: String = conexion.query_row(
        "SELECT strftime('%H:00', fecha_hora) FROM asistencias WHERE date(fecha_hora) BETWEEN date('now','localtime','-6 days') AND date('now','localtime') GROUP BY strftime('%H', fecha_hora) ORDER BY COUNT(*) DESC, strftime('%H', fecha_hora) ASC LIMIT 1",
        [], |row| row.get(0)
    ).unwrap_or_else(|_| "Sin datos".to_string());
    Ok(Metricas { total_clientes, asistencias_hoy, nuevos_semana, huella_activas, asistencias_ultimos_7_dias, dia_mas_concurrido, hora_mayor_afluencia })
}

/// Modifica únicamente datos administrativos; la huella queda intacta.
pub fn actualizar_cliente(conn: &Connection, id: i64, datos: ActualizacionCliente) -> Result<(), String> {
    if datos.nombre.trim().is_empty() || datos.nombre.chars().count() > 120 || datos.nombre.chars().any(char::is_control) {
        return Err("Nombre inválido".into());
    }
    for telefono in [&datos.telefono, &datos.telefono_emergencia] {
        if telefono.len() > 25 || !telefono.chars().all(|ch| ch.is_ascii_digit() || "+- ()".contains(ch)) {
            return Err("Teléfono inválido".into());
        }
    }
    for fecha in [&datos.fecha_inicio_membresia, &datos.fecha_fin_membresia] {
        if fecha.len() != 10 || chrono::NaiveDate::parse_from_str(fecha, "%Y-%m-%d").is_err() {
            return Err("Fecha de membresía inválida".into());
        }
    }
    if datos.fecha_fin_membresia < datos.fecha_inicio_membresia { return Err("La fecha final debe ser posterior a la inicial".into()); }
    let changed = conn.execute("UPDATE clientes SET nombre=?1, telefono=?2, telefono_emergencia=?3, fecha_inicio_membresia=?4, fecha_fin_membresia=?5 WHERE id=?6",
        params![datos.nombre.trim(), datos.telefono.trim(), datos.telefono_emergencia.trim(), datos.fecha_inicio_membresia, datos.fecha_fin_membresia, id]).map_err(|e| e.to_string())?;
    if changed == 0 { return Err("Cliente no encontrado".into()); }
    Ok(())
}

/// Desactivar revoca acceso biométrico, sin borrar el expediente.
pub fn establecer_cliente_activo(conn: &Connection, id: i64, activo: bool) -> Result<(), String> {
    let changed = conn.execute("UPDATE clientes SET activo=?1 WHERE id=?2", params![activo as i32, id]).map_err(|e| e.to_string())?;
    if changed == 0 { return Err("Cliente no encontrado".into()); }
    Ok(())
}

/// Eliminación irreversible de datos personales y biométricos.
/// Las tablas dependientes se eliminan por ON DELETE CASCADE dentro de la transacción.
pub fn eliminar_cliente_definitivamente(conn: &mut Connection, id: i64) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let changed = tx.execute("DELETE FROM clientes WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
    if changed == 0 { return Err("Cliente no encontrado".into()); }
    tx.commit().map_err(|e| e.to_string())
}
