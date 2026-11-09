//! Fixtures aisladas: nunca usan AppHandle, APPDATA ni gym.db.
use crate::{database, models::DatosCliente};
use rusqlite::Connection;
use std::path::PathBuf;

pub fn db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    database::inicializar_esquema(&conn).unwrap();
    conn
}

pub struct TempDb { pub dir: PathBuf, pub path: PathBuf }
impl TempDb {
    pub fn new() -> Self {
        let mut bytes = [0u8; 16];
        getrandom::getrandom(&mut bytes).unwrap();
        let suffix: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        let dir = std::env::temp_dir().join(format!("logym-test-{suffix}"));
        std::fs::create_dir(&dir).unwrap();
        Self { path: dir.join("fixture.sqlite"), dir }
    }
    pub fn connect(&self) -> Connection {
        let conn = Connection::open(&self.path).unwrap();
        conn.busy_timeout(std::time::Duration::from_secs(10)).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;").unwrap();
        conn
    }
}
impl Drop for TempDb {
    fn drop(&mut self) {
        // Solo se borra el directorio aleatorio creado por esta fixture.
        assert_eq!(self.dir.parent(), Some(std::env::temp_dir().as_path()));
        assert!(self.dir.file_name().unwrap().to_string_lossy().starts_with("logym-test-"));
        if let Err(error) = std::fs::remove_dir_all(&self.dir) {
            eprintln!("Fixture temporal no eliminada: {}: {error}", self.dir.display());
        }
    }
}

pub fn cliente(plan: i64) -> DatosCliente {
    DatosCliente { sin_huella: false, tipo_membresia_id: Some(plan), incluir_inscripcion: true,
        nombre: "Cliente sintético".into(), telefono: "5512345678".into(), telefono_emergencia: String::new(),
        fecha_registro: "2024-01-31".into(), fecha_inicio_membresia: "2024-01-31".into(),
        fecha_fin_membresia: "2024-02-29".into(), foto_base64: String::new(), template_huella: "ab".repeat(400) }
}

pub fn plan(conn: &Connection, nombre: &str) -> i64 {
    conn.query_row("SELECT id FROM tipos_membresia WHERE nombre=?1", [nombre], |r| r.get(0)).unwrap()
}

pub fn vigente(conn: &Connection) -> i64 {
    let id = crate::repositories::guardar_cliente(conn, cliente(plan(conn, "Mensual"))).unwrap();
    conn.execute("UPDATE clientes SET fecha_inicio_membresia=date('now','localtime','-10 days'), fecha_fin_membresia=date('now','localtime','+30 days') WHERE id=?1", [id]).unwrap();
    id
}

pub fn counts(conn: &Connection) -> (i64, i64, i64, i64) {
    conn.query_row("SELECT (SELECT count(*) FROM clientes),(SELECT count(*) FROM pagos),(SELECT count(*) FROM membresias_historial),(SELECT count(*) FROM asistencias)", [],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap()
}
