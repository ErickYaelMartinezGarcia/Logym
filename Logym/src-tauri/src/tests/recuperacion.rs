use super::*;
use crate::test_support::*;

#[cfg(windows)]
#[test]
fn importar_legacy_cifra_sin_cambiar_origen_y_rechaza_destino_existente() {
    let temp=TempDb::new();let source=temp.connect();
    source.execute_batch("CREATE TABLE clientes(id INTEGER PRIMARY KEY,nombre TEXT); INSERT INTO clientes VALUES(8,'Legacy sintético');").unwrap();
    drop(source);
    let antes=std::fs::read(&temp.path).unwrap();
    let target=temp.dir.join("cifrada.sqlite");
    importar_legacy(&temp.path,&target,"Admin-sintetico-2026").unwrap();
    assert_eq!(std::fs::read(&temp.path).unwrap(),antes);
    let conn=crate::database::abrir_cifrada(&target).unwrap();
    assert_eq!(conn.query_row("SELECT nombre FROM clientes WHERE id=8",[],|r|r.get::<_,String>(0)).unwrap(),"Legacy sintético");
    assert!(crate::repositories::registrar_asistencia(&conn,8).is_err());
    assert!(!std::fs::read(&target).unwrap().starts_with(b"SQLite format 3"));
    assert!(importar_legacy(&temp.path,&target,"Admin-sintetico-2026").is_err());
}

#[test]
fn exportacion_cifrada_incluye_wal_y_no_sobrescribe_respaldo() {
    let temp = TempDb::new();
    let conn = temp.connect();
    crate::database::inicializar_esquema(&conn).unwrap();
    vigente(&conn);
    let copy = temp.dir.join("copia.sqlcipher");
    let password = "Respaldo-sintetico-2026";
    exportar(&conn,&copy,password).unwrap();
    let bytes = std::fs::read(&copy).unwrap();
    assert!(!bytes.starts_with(b"SQLite format 3"));
    assert!(abrir_clave(&copy,"incorrecta").is_err());
    let saved = abrir_clave(&copy,password).unwrap();
    assert_eq!(counts(&saved),counts(&conn));
    assert_eq!(saved.query_row("PRAGMA user_version",[],|r|r.get::<_,i64>(0)).unwrap(),1);
    assert!(exportar(&conn,&copy,password).is_err());
    assert_eq!(std::fs::read(&copy).unwrap(),bytes);
}

#[cfg(windows)]
#[test]
fn restauracion_portable_y_rechazo_de_credenciales_sin_cambios() {
    let source = TempDb::new();
    let conn = crate::database::abrir_cifrada(&source.path).unwrap();
    crate::database::inicializar_esquema(&conn).unwrap();
    let admin = "Administrador-sintetico-2026";
    crate::membresias::configurar_administrador(&conn,admin).unwrap();
    vigente(&conn);
    let expected = counts(&conn);
    let backup = source.dir.join("portatil.sqlcipher");
    drop(conn);
    respaldar(&source.path,&backup,"Respaldo-sintetico-2026",admin).unwrap();
    let target = TempDb::new();
    assert!(restaurar(&backup,&target.path,"incorrecta",admin).is_err());
    assert!(!target.path.exists());
    restaurar(&backup,&target.path,"Respaldo-sintetico-2026",admin).unwrap();
    let conn = crate::database::abrir_cifrada(&target.path).unwrap();
    assert_eq!(counts(&conn),expected);
    vigente(&conn);
    let before = counts(&conn);
    drop(conn);
    assert!(restaurar(&backup,&target.path,"Respaldo-sintetico-2026","incorrecto").is_err());
    assert_eq!(counts(&crate::database::abrir_cifrada(&target.path).unwrap()),before);
    restaurar(&backup,&target.path,"Respaldo-sintetico-2026",admin).unwrap();
    assert_eq!(counts(&crate::database::abrir_cifrada(&target.path).unwrap()),expected);
}

#[test]
fn legacy_fechas_ausentes_no_conceden_acceso_y_migracion_es_atomica() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON; CREATE TABLE clientes(id INTEGER PRIMARY KEY,nombre TEXT); INSERT INTO clientes VALUES(1,'Legacy'); CREATE TABLE asistencias(id INTEGER PRIMARY KEY,cliente_id INTEGER,fecha_hora TEXT); INSERT INTO asistencias VALUES(2,999,'2020-01-01');").unwrap();
    assert!(crate::database::inicializar_esquema(&conn).is_err());
    assert_eq!(conn.query_row("SELECT count(*) FROM asistencias",[],|r|r.get::<_,i64>(0)).unwrap(),1);
    assert_eq!(conn.query_row("SELECT count(*) FROM pragma_table_info('clientes')",[],|r|r.get::<_,i64>(0)).unwrap(),2);
    conn.execute("UPDATE asistencias SET cliente_id=1",[]).unwrap();
    crate::database::inicializar_esquema(&conn).unwrap();
    assert_eq!(crate::repositories::listar_clientes(&conn).unwrap().len(),1);
    assert!(crate::repositories::registrar_asistencia(&conn,1).is_err());
    assert!(conn.execute("INSERT INTO asistencias(cliente_id) VALUES(999)",[]).is_err());
}

fn contenido_completo(db:&Connection)->Vec<(String,Vec<Vec<rusqlite::types::Value>>)> {
    let tablas:Vec<String>=db.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").unwrap().query_map([],|r|r.get(0)).unwrap().collect::<Result<_,_>>().unwrap();
    tablas.into_iter().map(|t|{let mut stmt=db.prepare(&format!("SELECT * FROM \"{}\" ORDER BY rowid",t.replace('"',"\"\""))).unwrap();let columnas=stmt.column_count();let rows=stmt.query_map([],|r|(0..columnas).map(|c|r.get(c)).collect::<Result<Vec<rusqlite::types::Value>,_>>()).unwrap().collect::<Result<Vec<_>,_>>().unwrap();(t,rows)}).collect()
}

#[cfg(windows)]
#[test]
fn respaldo_restauracion_integra_todas_las_tablas_con_clave_dpapi_nueva() {
    let source=TempDb::new();let db=crate::database::abrir_cifrada(&source.path).unwrap();crate::database::inicializar_esquema(&db).unwrap();
    let admin="Admin-completo-sintetico-2026";let password="Backup-completo-sintetico-2026";crate::membresias::configurar_administrador(&db,admin).unwrap();
    let id=vigente(&db);db.execute("UPDATE clientes SET foto_base64='data:image/jpeg;base64,U1lOVEhFVElD',telefono_emergencia='5511112233' WHERE id=?1",[id]).unwrap();
    crate::repositories::registrar_asistencia(&db,id).unwrap();crate::repositories::renovar_membresia(&db,id,plan(&db,"Mensual")).unwrap();db.execute("UPDATE tipos_membresia SET precio_centavos=99999 WHERE id=?1",[plan(&db,"Mensual")]).unwrap();
    let expected=contenido_completo(&db);let original_key=crate::database::clave_db(&source.path).unwrap();drop(db);
    let backup=source.dir.join("portable.sqlcipher");respaldar(&source.path,&backup,password,admin).unwrap();let backup_bytes=std::fs::read(&backup).unwrap();
    let target=TempDb::new();let license=target.dir.join("license.txt");std::fs::write(&license,b"synthetic destination license").unwrap();
    restaurar(&backup,&target.path,password,admin).unwrap();assert_ne!(crate::database::clave_db(&target.path).unwrap(),original_key);
    let db=crate::database::abrir_cifrada(&target.path).unwrap();comprobar(&db).unwrap();assert!(!db.prepare("PRAGMA foreign_key_check").unwrap().exists([]).unwrap());assert_eq!(contenido_completo(&db),expected);
    vigente(&db);let before=contenido_completo(&db);drop(db);
    assert!(restaurar(&backup,&target.path,"Password-incorrecto-2026",admin).is_err());assert_eq!(contenido_completo(&crate::database::abrir_cifrada(&target.path).unwrap()),before);
    let corrupt=source.dir.join("corrupto.sqlcipher");std::fs::write(&corrupt,b"not a backup").unwrap();assert!(restaurar(&corrupt,&target.path,password,admin).is_err());assert_eq!(contenido_completo(&crate::database::abrir_cifrada(&target.path).unwrap()),before);
    let orphan=source.dir.join("huerfano.sqlcipher");std::fs::write(&orphan,&backup_bytes).unwrap();let orphan_db=abrir_clave(&orphan,password).unwrap();orphan_db.execute_batch("PRAGMA foreign_keys=OFF; INSERT INTO asistencias(cliente_id,fecha_hora) VALUES(99999,'2026-01-01');").unwrap();drop(orphan_db);
    assert!(restaurar(&orphan,&target.path,password,admin).unwrap_err().contains("referencias"));assert_eq!(contenido_completo(&crate::database::abrir_cifrada(&target.path).unwrap()),before);
    let future=source.dir.join("futuro.sqlcipher");std::fs::write(&future,&backup_bytes).unwrap();let future_db=abrir_clave(&future,password).unwrap();future_db.pragma_update(None,"user_version",99).unwrap();drop(future_db);
    assert!(restaurar(&future,&target.path,password,admin).unwrap_err().contains("más nueva"));assert_eq!(contenido_completo(&crate::database::abrir_cifrada(&target.path).unwrap()),before);
    restaurar(&backup,&target.path,password,admin).unwrap();assert_eq!(contenido_completo(&crate::database::abrir_cifrada(&target.path).unwrap()),expected);
    let snapshots:Vec<_>=std::fs::read_dir(&target.dir).unwrap().map(|p|p.unwrap().path()).filter(|p|p.to_string_lossy().contains("pre-restauracion-")).collect();assert_eq!(snapshots.len(),1);
    let key=crate::database::clave_db(&target.path).unwrap();let previous=abrir_clave(&snapshots[0],&format!("x'{key}'")).unwrap();assert_eq!(contenido_completo(&previous),before);
    assert_eq!(std::fs::read(&license).unwrap(),b"synthetic destination license");assert_eq!(std::fs::read(&backup).unwrap(),backup_bytes);
}
