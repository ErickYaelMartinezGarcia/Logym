use super::*;
use crate::test_support::*;

fn conteos(conn: &Connection) -> (i64, i64, i64) {
    conn.query_row("SELECT (SELECT count(*) FROM visitas_ocasionales),(SELECT count(*) FROM pagos_visitas),(SELECT count(*) FROM asistencias_visitas)", [], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap()
}

#[test]
fn visita_con_y_sin_nombre_es_atomica_fija_y_no_crea_miembros() {
    let db = db();
    for (i, nombre) in ["", " Visitante sintético "].iter().enumerate() {
        let id = format!("{i:032x}");
        let recibo = registrar(&db, &id, nombre).unwrap();
        assert_eq!(recibo.total_centavos, 5000);
        assert_eq!(recibo.id, id);
        let guardado: Option<String> = db.query_row("SELECT nombre FROM visitas_ocasionales WHERE id=?1",[&id],|r|r.get(0)).unwrap();
        assert_eq!(guardado.as_deref(), if nombre.is_empty() { None } else { Some(nombre.trim()) });
        registrar(&db, &id, nombre).unwrap();
    }
    assert_eq!(conteos(&db), (2,2,2));
    assert_eq!(counts(&db), (0,0,0,0));
    assert!(db.execute("UPDATE pagos_visitas SET total_centavos=1",[]).is_err());
    assert!(registrar(&db, &format!("{:032x}",1), "Otro nombre").is_err());
    assert!(registrar(&db, "no-es-un-id", "").is_err());
    assert!(registrar(&db, &"f".repeat(32), "Nombre\ninválido").is_err());
    assert_eq!(conteos(&db), (2,2,2));
}

#[test]
fn fallos_de_pago_o_asistencia_revierten_toda_la_visita() {
    for tabla in ["pagos_visitas", "asistencias_visitas"] {
        let db = db();
        db.execute_batch(&format!("CREATE TRIGGER fallo BEFORE INSERT ON {tabla} BEGIN SELECT RAISE(ABORT,'fallo sintético'); END;")).unwrap();
        assert!(registrar(&db, &"a".repeat(32), "").is_err());
        assert_eq!(conteos(&db), (0,0,0));
        db.execute_batch("DROP TRIGGER fallo").unwrap();
        registrar(&db, &"a".repeat(32), "").unwrap();
        assert_eq!(conteos(&db), (1,1,1));
    }
}

#[test]
fn envios_concurrentes_y_reapertura_no_duplican_y_otro_id_cobra_otra_entrada() {
    let temp = TempDb::new();
    crate::database::inicializar_esquema(&temp.connect()).unwrap();
    let handles: Vec<_> = (0..4).map(|_| {
        let path = temp.path.clone();
        std::thread::spawn(move || {
            let db = Connection::open(path).unwrap();
            db.busy_timeout(std::time::Duration::from_secs(10)).unwrap();
            registrar(&db, &"b".repeat(32), "Sintético").unwrap();
        })
    }).collect();
    for handle in handles { handle.join().unwrap(); }
    let db = temp.connect();
    crate::database::inicializar_esquema(&db).unwrap();
    assert_eq!(conteos(&db), (1,1,1));
    registrar(&db, &"b".repeat(32), "Sintético").unwrap();
    registrar(&db, &"c".repeat(32), "Sintético").unwrap();
    assert_eq!(conteos(&db), (2,2,2));
    assert_eq!(crate::repositories::obtener_metricas(&db).unwrap().asistencias_hoy, 2);
}

#[test]
fn visitas_se_integran_a_finanzas_y_asistencias_sin_ser_miembros() {
    let db = db();
    vigente(&db); // Mensual 350 + inscripción 150, sin asistencia todavía.
    for i in 0..2 { registrar(&db, &format!("{i:032x}"), "").unwrap(); }
    let hoy = chrono::Local::now().format("%Y-%m-%d").to_string();
    let reporte = crate::membresias::ingresos_rango(&db, &hoy, &hoy).unwrap();
    assert_eq!((reporte.total_centavos,reporte.membresias_centavos,reporte.inscripciones_centavos,reporte.visitas_centavos,reporte.numero_pagos),(60000,35000,15000,10000,3));
    assert_eq!(reporte.serie.iter().map(|p|p.total_centavos).sum::<i64>(), 60000);
    for periodo in ["dia","semana","mes","anual"] {
        let resumen = crate::membresias::ingresos(&db, periodo).unwrap();
        assert_eq!(resumen.total_centavos, resumen.membresias_centavos + resumen.inscripciones_centavos + resumen.visitas_centavos);
        assert_eq!(resumen.total_centavos,60000);
    }
    let metricas = crate::repositories::obtener_metricas(&db).unwrap();
    assert_eq!((metricas.total_clientes,metricas.asistencias_hoy,metricas.asistencias_ultimos_7_dias),(1,2,2));
    for periodo in ["dia","semana","mes"] {
        assert_eq!(crate::membresias::asistencias_periodo(&db,periodo).unwrap(),2);
        assert_eq!(crate::membresias::asistencias_diarias(&db,periodo).unwrap().iter().map(|p|p.total).sum::<i64>(),2);
    }
    assert_eq!(crate::membresias::asistencias_por_hora_hoy(&db).unwrap().iter().map(|p|p.total).sum::<i64>(),2);
    db.execute("UPDATE pagos_visitas SET fecha_pago='2024-01-31 23:59:59' WHERE visita_id=?1",[format!("{:032x}",0)]).unwrap();
    db.execute("UPDATE pagos_visitas SET fecha_pago='2024-02-01 00:00:00' WHERE visita_id=?1",[format!("{:032x}",1)]).unwrap();
    for (inicio,fin,esperado) in [("2024-01-31","2024-01-31",5000),("2024-01-01","2024-01-31",5000),("2024-01-01","2024-12-31",10000)] {
        let r = crate::membresias::ingresos_rango(&db,inicio,fin).unwrap();
        assert_eq!((r.total_centavos,r.visitas_centavos,r.membresias_centavos,r.inscripciones_centavos),(esperado,esperado,0,0));
    }
}

#[cfg(windows)]
#[test]
fn visitas_persisten_en_sqlcipher_con_dpapi_y_reintento_tras_reabrir() {
    let temp = TempDb::new();
    let db = crate::database::abrir_cifrada(&temp.path).unwrap();
    crate::database::inicializar_esquema(&db).unwrap();
    registrar(&db,&"d".repeat(32),"Sintético").unwrap();
    drop(db);
    let db = crate::database::abrir_cifrada(&temp.path).unwrap();
    crate::database::inicializar_esquema(&db).unwrap();
    registrar(&db,&"d".repeat(32),"Sintético").unwrap();
    assert_eq!(conteos(&db),(1,1,1));
    assert!(!std::fs::read(&temp.path).unwrap().starts_with(b"SQLite format 3"));
}
