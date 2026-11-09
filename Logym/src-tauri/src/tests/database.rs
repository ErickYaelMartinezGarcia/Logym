use super::*;
use crate::test_support::*;

#[test]
fn migracion_version_uno_conserva_catalogo_pagos_y_agrega_dias_y_visitas() {
    let temp=TempDb::new();let db=temp.connect();inicializar_esquema(&db).unwrap();
    vigente(&db);
    db.execute("UPDATE tipos_membresia SET precio_centavos=12345 WHERE nombre='Mensual'",[]).unwrap();
    let antes=counts(&db);
    db.execute_batch("DROP VIEW ingresos_todos; DROP VIEW asistencias_todas; DROP TABLE pagos_visitas; DROP TABLE asistencias_visitas; DROP TABLE visitas_ocasionales; ALTER TABLE tipos_membresia DROP COLUMN duracion_dias; PRAGMA user_version=1;").unwrap();
    drop(db);
    let db=temp.connect();inicializar_esquema(&db).unwrap();inicializar_esquema(&db).unwrap();
    assert_eq!(counts(&db),antes);
    assert_eq!(db.query_row("PRAGMA user_version",[],|r|r.get::<_,i64>(0)).unwrap(),VERSION_ESQUEMA);
    assert_eq!(db.query_row("SELECT duracion_meses,duracion_dias,precio_centavos FROM tipos_membresia WHERE nombre='Mensual'",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,Option<i64>>(1)?,r.get::<_,i64>(2)?))).unwrap(),(1,None,12345));
    assert_eq!(db.query_row("SELECT membresia_centavos FROM pagos",[],|r|r.get::<_,i64>(0)).unwrap(),35000);
    crate::visitas::registrar(&db,&"f".repeat(32),"").unwrap();
    assert_eq!(db.query_row("PRAGMA integrity_check",[],|r|r.get::<_,String>(0)).unwrap(),"ok");
    assert!(!db.prepare("PRAGMA foreign_key_check").unwrap().exists([]).unwrap());
}

#[test]
fn esquema_real_restricciones_fk_importes_y_catalogo() {
    let db=db();let id=vigente(&db);
    assert_eq!(db.query_row("PRAGMA foreign_keys",[],|r|r.get::<_,i64>(0)).unwrap(),1);
    for sql in [
        "INSERT INTO asistencias(cliente_id) VALUES(-999)",
        "INSERT INTO tipos_membresia(nombre,duracion_meses,precio_centavos) VALUES('negativo',1,-1)",
        "INSERT INTO tipos_membresia(nombre,duracion_meses,precio_centavos) VALUES('cero',0,1)",
        "INSERT INTO tipos_membresia(nombre,duracion_meses,precio_centavos,activo) VALUES('activo',1,1,2)",
        "INSERT INTO configuracion_cobros(id,inscripcion_centavos) VALUES(2,0)",
        "INSERT INTO clientes(nombre) VALUES(NULL)",
    ] {assert!(db.execute(sql,[]).is_err(),"{sql}");}
    assert!(db.execute("INSERT INTO pagos(cliente_id,concepto_membresia,membresia_centavos,total_centavos,tipo) VALUES(?1,'Prueba',100,101,'alta')",[id]).is_err());
    assert!(db.execute("INSERT INTO pagos(cliente_id,concepto_membresia,membresia_centavos,total_centavos,tipo) VALUES(?1,'Prueba',100,100,'otro')",[id]).is_err());
    assert!(db.query_row("PRAGMA foreign_key_check",[],|_|Ok(())).optional().unwrap().is_none());
}
use rusqlite::OptionalExtension;

#[cfg(windows)]
#[test]
fn cliente_sin_huella_persiste_en_sqlcipher_con_dpapi() {
    let temp = TempDb::new();
    let db = abrir_cifrada(&temp.path).unwrap();
    inicializar_esquema(&db).unwrap();
    let mut datos = cliente(plan(&db, "Mensual"));
    datos.sin_huella = true;
    datos.template_huella.clear();
    let id = crate::repositories::guardar_cliente_sin_duplicar(&db, datos, |_,_| panic!("SDK")).unwrap();
    drop(db);
    let db = abrir_cifrada(&temp.path).unwrap();
    inicializar_esquema(&db).unwrap();
    let template: Option<String> = db.query_row("SELECT template_huella FROM clientes WHERE id=?1", [id], |r| r.get(0)).unwrap();
    assert_eq!(template, None);
    assert_eq!(counts(&db), (1,1,1,0));
    assert!(!std::fs::read(&temp.path).unwrap().starts_with(b"SQLite format 3"));
}

#[test]
fn esquema_idempotente_conserva_precios_configurados_datos_y_hash() {
    let db=db();let id=vigente(&db);crate::repositories::registrar_asistencia(&db,id).unwrap();
    db.execute("UPDATE tipos_membresia SET precio_centavos=12345 WHERE nombre='Mensual'",[]).unwrap();
    db.execute("UPDATE configuracion_cobros SET inscripcion_centavos=0",[]).unwrap();
    let before=counts(&db);for _ in 0..3 {inicializar_esquema(&db).unwrap();}
    assert_eq!(counts(&db),before);assert_eq!(crate::membresias::inscripcion(&db).unwrap(),0);
    assert_eq!(db.query_row("SELECT precio_centavos FROM tipos_membresia WHERE nombre='Mensual'",[],|r|r.get::<_,i64>(0)).unwrap(),12345);
}

#[test]
fn persistencia_cerrar_y_reabrir_esquema_clientes_pagos_asistencias() {
    let temp=TempDb::new();let db=temp.connect();inicializar_esquema(&db).unwrap();let id=vigente(&db);crate::repositories::registrar_asistencia(&db,id).unwrap();drop(db);
    let db=temp.connect();inicializar_esquema(&db).unwrap();assert_eq!(counts(&db),(1,1,1,1));
    assert_eq!(db.query_row("PRAGMA integrity_check",[],|r|r.get::<_,String>(0)).unwrap(),"ok");
}

#[test]
fn migracion_catalogo_antiguo_agrega_solo_alta_sin_reemplazar_precios() {
    let db=Connection::open_in_memory().unwrap();db.execute_batch("CREATE TABLE tipos_membresia(id INTEGER PRIMARY KEY,nombre TEXT UNIQUE,duracion_meses INTEGER,precio_centavos INTEGER,activo INTEGER DEFAULT 1); INSERT INTO tipos_membresia VALUES(1,'Pareja',1,12345,1);").unwrap();
    inicializar_esquema(&db).unwrap();assert_eq!(db.query_row("SELECT precio_centavos,solo_alta FROM tipos_membresia",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?))).unwrap(),(12345,1));
}

#[test]
fn defecto_migracion_cliente_legacy_sin_columnas_de_fechas() {
    let db=Connection::open_in_memory().unwrap();db.execute_batch("CREATE TABLE clientes(id INTEGER PRIMARY KEY,nombre TEXT); INSERT INTO clientes VALUES(1,'Sintético');").unwrap();
    let result=inicializar_esquema(&db);assert!(result.is_ok(),"Migración falló: {result:?}");
}

#[test]
fn defecto_migracion_asistencias_legacy_no_debe_dejar_huerfanos() {
    let mut db=Connection::open_in_memory().unwrap();db.execute_batch("PRAGMA foreign_keys=ON; CREATE TABLE clientes(id INTEGER PRIMARY KEY,nombre TEXT,fecha_inicio_membresia TEXT,fecha_fin_membresia TEXT); CREATE TABLE asistencias(id INTEGER PRIMARY KEY,cliente_id INTEGER,fecha_hora TEXT); INSERT INTO clientes VALUES(1,'Sintético','2024-01-01','2024-02-01'); INSERT INTO asistencias VALUES(1,1,'2024-01-01');").unwrap();
    inicializar_esquema(&db).unwrap();crate::repositories::eliminar_cliente_definitivamente(&mut db,1).unwrap();assert_eq!(db.query_row("SELECT count(*) FROM asistencias",[],|r|r.get::<_,i64>(0)).unwrap(),0,"Asistencia huérfana después de migrar/eliminar");
}

#[cfg(windows)]
#[test]
fn sqlcipher_produccion_dpapi_reapertura_clave_ausente_e_incorrecta() {
    let temp=TempDb::new();let db=abrir_cifrada(&temp.path).unwrap();
    let version:String=db.query_row("PRAGMA cipher_version",[],|r|r.get(0)).unwrap();assert!(!version.is_empty());
    inicializar_esquema(&db).unwrap();let id=vigente(&db);crate::repositories::registrar_asistencia(&db,id).unwrap();drop(db);
    let bytes=std::fs::read(&temp.path).unwrap();assert!(!bytes.starts_with(b"SQLite format 3"));
    let no_key=Connection::open(&temp.path).unwrap();assert!(no_key.query_row("SELECT count(*) FROM sqlite_master",[],|r|r.get::<_,i64>(0)).is_err());drop(no_key);
    let wrong=Connection::open(&temp.path).unwrap();wrong.execute_batch("PRAGMA key='clave-sintetica-incorrecta';").unwrap();assert!(wrong.query_row("SELECT count(*) FROM sqlite_master",[],|r|r.get::<_,i64>(0)).is_err());drop(wrong);
    let reopened=abrir_cifrada(&temp.path).unwrap();assert_eq!(counts(&reopened),(1,1,1,1));assert_eq!(reopened.query_row("PRAGMA foreign_keys",[],|r|r.get::<_,i64>(0)).unwrap(),1);drop(reopened);
    std::fs::write(temp.path.with_extension("key.dpapi"),b"DPAPI-corrupto-sintetico").unwrap();assert!(abrir_cifrada(&temp.path).is_err());
}

#[cfg(windows)]
#[test]
fn base_existente_sin_archivo_clave_no_se_modifica() {
    let temp=TempDb::new();let db=temp.connect();db.execute_batch("CREATE TABLE marcador(valor TEXT); INSERT INTO marcador VALUES('sintético');").unwrap();drop(db);
    let before=std::fs::read(&temp.path).unwrap();assert!(abrir_cifrada(&temp.path).is_err());assert_eq!(std::fs::read(&temp.path).unwrap(),before);assert!(!temp.path.with_extension("key.dpapi").exists());
}

#[cfg(windows)]
#[test]
fn sqlcipher_concurrencia_en_base_cifrada_persistente() {
    let temp=TempDb::new();let db=abrir_cifrada(&temp.path).unwrap();inicializar_esquema(&db).unwrap();let id=vigente(&db);drop(db);
    let connections:Vec<_>=(0..6).map(|_|{let c=abrir_cifrada(&temp.path).unwrap();c.busy_timeout(std::time::Duration::from_secs(10)).unwrap();c}).collect();
    let barrier=std::sync::Arc::new(std::sync::Barrier::new(6));
    let handles:Vec<_>=connections.into_iter().map(|db|{let b=barrier.clone();std::thread::spawn(move||{b.wait();crate::repositories::registrar_asistencia(&db,id)})}).collect();
    let results:Vec<_>=handles.into_iter().map(|h|h.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r|r.is_ok()).count(),1,"{results:?}");
    let db=abrir_cifrada(&temp.path).unwrap();assert_eq!(counts(&db),(1,1,1,1));
}

#[cfg(windows)]
#[test]
fn dpapi_ida_vuelta_y_rechazo_de_datos_corruptos() {
    let raw=b"material-de-prueba-sin-datos-reales";let encrypted=dpapi(raw,true).unwrap();assert_ne!(encrypted,raw);assert_eq!(dpapi(&encrypted,false).unwrap(),raw);assert!(dpapi(b"invalido",false).is_err());
}
