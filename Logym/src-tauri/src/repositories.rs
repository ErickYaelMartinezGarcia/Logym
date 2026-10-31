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

pub fn listar_clientes_membresia(conexion: &Connection) -> Result<Vec<ClienteMembresia>, String> {
    let mut stmt = conexion.prepare("SELECT id, nombre, telefono, fecha_fin_membresia, CAST(julianday(fecha_fin_membresia) - julianday('now') AS INTEGER) as dias FROM clientes ORDER BY dias ASC").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| {
        let dias: i64 = r.get(4)?;
        let estado = if dias < 0 { "vencida".to_string() } else if dias <= 3 { "por_vencer".to_string() } else { "activa".to_string() };
        Ok(ClienteMembresia{ id: r.get(0)?, nombre: r.get(1)?, telefono: r.get(2)?, fecha_fin_membresia: r.get(3)?, dias_restantes: dias, estado })
    }).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

// ESTA ES LA BUENA - solo trae datos, no compara
pub fn listar_con_huella(conexion: &Connection) -> Result<Vec<(i64, String, String, String)>, String> {
    let mut stmt = conexion.prepare("SELECT id, nombre, telefono, template_huella FROM clientes WHERE template_huella IS NOT NULL AND template_huella != ''").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn registrar_asistencia(conexion: &Connection, cliente_id: i64) -> Result<String, String> {
    let n = conexion.execute("INSERT INTO asistencias (cliente_id, fecha_hora) SELECT ?1, datetime('now','localtime') WHERE NOT EXISTS (SELECT 1 FROM asistencias WHERE cliente_id = ?1 AND date(fecha_hora) = date('now','localtime'))", params![cliente_id]).map_err(|e| e.to_string())?;
    if n == 0 { return Err("Ya registró asistencia hoy".into()) }
    Ok("Asistencia registrada".into())
}

pub fn renovar_membresia(conexion: &Connection, cliente_id: i64) -> Result<String, String> {
    let tx = conexion.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute("UPDATE clientes SET fecha_inicio_membresia = date('now','localtime'), fecha_fin_membresia = date('now','localtime','+30 day') WHERE id = ?1", params![cliente_id]).map_err(|e| e.to_string())?;
    tx.execute("INSERT INTO membresias_historial (cliente_id, fecha_inicio, fecha_fin, fecha_pago, tipo) VALUES (?1, date('now','localtime'), date('now','localtime','+30 day'), datetime('now','localtime'), 'renovacion')", params![cliente_id]).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok("Renovada 30 días".into())
}

pub fn obtener_metricas(conexion: &Connection) -> Result<Metricas, String> {
    let hoy: i64 = conexion.query_row("SELECT COUNT(*) FROM asistencias WHERE date(fecha_hora)=date('now','localtime')", [], |r| r.get(0)).map_err(|e| e.to_string())?;
    let sem: i64 = conexion.query_row("SELECT COUNT(*) FROM asistencias WHERE date(fecha_hora) >= date('now','-7 day')", [], |r| r.get(0)).map_err(|e| e.to_string())?;
    Ok(Metricas{ entradas_hoy: hoy, entradas_7_dias: sem, dia_pico: "Lunes".into(), hora_pico: "18:00".into() })
}