use rusqlite::{params, Connection, OptionalExtension};

const PRECIO_CENTAVOS: i64 = 5000;

#[derive(Debug, serde::Serialize)]
pub struct Visita {
    pub id: String,
    pub fecha_hora: String,
    pub total_centavos: i64,
}

/// El ID identifica una confirmación, no a una persona ni a un pase reutilizable.
/// Repetir la petición devuelve su recibo; otra entrada necesita una operación nueva.
pub fn registrar(conn: &Connection, operacion_id: &str, nombre: &str) -> Result<Visita, String> {
    if operacion_id.len() != 32 || !operacion_id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Identificador de visita inválido".into());
    }
    let nombre = nombre.trim();
    if nombre.chars().count() > 120 || nombre.chars().any(char::is_control) {
        return Err("Nombre de visitante inválido".into());
    }
    let id = operacion_id.to_ascii_lowercase();
    let tx = rusqlite::Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Immediate).map_err(|e|e.to_string())?;
    let existente = tx.query_row("SELECT nombre,fecha_hora FROM visitas_ocasionales WHERE id=?1", [&id],
        |r| Ok((r.get::<_,Option<String>>(0)?.unwrap_or_default(), r.get::<_,String>(1)?))).optional().map_err(|e|e.to_string())?;
    if let Some((anterior, fecha_hora)) = existente {
        // Un reintento tras reiniciar conserva solo el ID en el navegador.
        // El nombre vacío permite consultar el recibo sin reescribir sus datos.
        if !nombre.is_empty() && anterior != nombre { return Err("Esta operación ya fue registrada con otros datos".into()); }
        return Ok(Visita { id, fecha_hora, total_centavos: PRECIO_CENTAVOS });
    }
    tx.execute("INSERT INTO visitas_ocasionales(id,nombre) VALUES(?1,?2)", params![id, if nombre.is_empty() { None } else { Some(nombre) }]).map_err(|e|e.to_string())?;
    let fecha_hora: String = tx.query_row("SELECT fecha_hora FROM visitas_ocasionales WHERE id=?1", [&id], |r|r.get(0)).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO pagos_visitas(visita_id,total_centavos,fecha_pago) VALUES(?1,?2,?3)", params![id,PRECIO_CENTAVOS,fecha_hora]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO asistencias_visitas(visita_id,fecha_hora) VALUES(?1,?2)", params![id,fecha_hora]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    Ok(Visita { id, fecha_hora, total_centavos: PRECIO_CENTAVOS })
}

#[cfg(test)]
#[path = "tests/visitas.rs"]
mod tests;
