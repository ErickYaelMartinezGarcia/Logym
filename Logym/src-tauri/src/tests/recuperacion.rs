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
