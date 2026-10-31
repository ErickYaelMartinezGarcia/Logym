use rusqlite::{params, Connection};
use serde::Serialize;

#[derive(Serialize)]
pub struct TipoMembresia {
    pub id: i64,
    pub nombre: String,
    pub duracion_meses: i64,
    pub precio_centavos: i64,
}

#[derive(Serialize)]
pub struct ResumenIngresos {
    pub periodo: String,
    pub total_centavos: i64,
    pub membresias_centavos: i64,
    pub inscripciones_centavos: i64,
    pub numero_pagos: i64,
}

pub fn listar(conn: &Connection) -> Result<Vec<TipoMembresia>, String> {
    let mut stmt = conn.prepare("SELECT id,nombre,duracion_meses,precio_centavos FROM tipos_membresia WHERE activo=1 ORDER BY nombre")
        .map_err(|e| e.to_string())?;
    stmt.query_map([], |r| Ok(TipoMembresia {
        id: r.get(0)?, nombre: r.get(1)?, duracion_meses: r.get(2)?, precio_centavos: r.get(3)?,
    })).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())
}

pub fn inscripcion(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT inscripcion_centavos FROM configuracion_cobros WHERE id=1", [], |r| r.get(0))
        .map_err(|e| e.to_string())
}

pub fn ingresos(conn: &Connection, periodo: &str) -> Result<ResumenIngresos, String> {
    let inicio = match periodo {
        "dia" => "date('now','localtime')",
        "semana" => "date('now','localtime','-6 days')",
        "mes" => "date('now','localtime','start of month')",
        _ => return Err("Periodo inválido".into()),
    };
    let sql = format!("SELECT COALESCE(SUM(total_centavos),0),COALESCE(SUM(membresia_centavos),0),COALESCE(SUM(inscripcion_centavos),0),COUNT(*) FROM pagos WHERE date(fecha_pago) BETWEEN {inicio} AND date('now','localtime')");
    conn.query_row(&sql, params![], |r| Ok(ResumenIngresos {
        periodo: periodo.to_string(), total_centavos: r.get(0)?,
        membresias_centavos: r.get(1)?, inscripciones_centavos: r.get(2)?,
        numero_pagos: r.get(3)?,
    })).map_err(|e| e.to_string())
}
