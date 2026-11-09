    use super::*;
    fn sin_huella(plan: i64) -> DatosCliente {
        let mut d = crate::test_support::cliente(plan);
        d.sin_huella = true;
        d.template_huella.clear();
        d.fecha_inicio_membresia = chrono::Local::now().format("%Y-%m-%d").to_string();
        d.fecha_fin_membresia = (chrono::Local::now() + chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
        d
    }

    #[test]
    fn alta_sin_huella_persiste_null_y_conserva_base_existente_al_reabrir() {
        use crate::test_support::*;
        let temp = TempDb::new();
        let db = temp.connect();
        crate::database::inicializar_esquema(&db).unwrap();
        let anterior = vigente(&db);
        registrar_asistencia(&db, anterior).unwrap();
        let snapshot = |conn: &Connection| {
            let mut tablas = Vec::new();
            for tabla in ["clientes", "pagos", "membresias_historial", "asistencias"] {
                let col = if tabla == "clientes" { "id" } else { "cliente_id" };
                let mut stmt = conn.prepare(&format!("SELECT * FROM {tabla} WHERE {col}=?1 ORDER BY id")).unwrap();
                let n = stmt.column_count();
                tablas.push(stmt.query_map([anterior], |r| (0..n).map(|i| r.get::<_, rusqlite::types::Value>(i)).collect::<rusqlite::Result<Vec<_>>>()).unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap());
            }
            tablas
        };
        let antes = snapshot(&db);
        let p = plan(&db, "Mensual");
        for _ in 0..2 {
            guardar_cliente_sin_duplicar(&db, sin_huella(p), |_, _| panic!("SDK innecesario")).unwrap();
        }
        assert_eq!(snapshot(&db), antes);
        drop(db);
        let db = temp.connect();
        crate::database::inicializar_esquema(&db).unwrap();
        assert_eq!(snapshot(&db), antes);
        assert_eq!(counts(&db), (3,3,3,1));
        assert_eq!(db.query_row("SELECT count(*) FROM clientes WHERE template_huella IS NULL", [], |r| r.get::<_,i64>(0)).unwrap(), 2);
        assert_eq!(db.query_row("SELECT sum(total_centavos) FROM pagos", [], |r| r.get::<_,i64>(0)).unwrap(), 150000);
        assert_eq!(db.query_row("PRAGMA user_version", [], |r| r.get::<_,i64>(0)).unwrap(), 1);
    }

    #[test]
    fn asistencia_sin_huella_reutiliza_restricciones_y_duplicados_entre_metodos() {
        use crate::test_support::*;
        let db = crate::test_support::db();
        let id = guardar_cliente_sin_duplicar(&db, sin_huella(plan(&db,"Mensual")), |_,_| panic!("SDK")).unwrap();
        registrar_asistencia(&db, id).unwrap();
        assert!(registrar_asistencia(&db,id).unwrap_err().contains("Ya registró"));
        let bio = vigente(&db);
        // El supervisor biométrico y el comando manual llaman al mismo repositorio.
        registrar_asistencia(&db,bio).unwrap();
        assert!(registrar_asistencia(&db,bio).unwrap_err().contains("Ya registró"));
        for (inicio, fin, activo, error) in [
            ("-30 days","-1 day",1,"vencida"),
            ("+1 day","+30 days",1,"pendiente"),
            ("-1 day","+30 days",0,"desactivado"),
        ] {
            db.execute("UPDATE clientes SET fecha_inicio_membresia=date('now','localtime',?1),fecha_fin_membresia=date('now','localtime',?2),activo=?3 WHERE id=?4",params![inicio,fin,activo,id]).unwrap();
            assert!(registrar_asistencia(&db,id).unwrap_err().contains(error));
        }
        assert_eq!(counts(&db), (2,2,2,2));
    }

    #[test]
    fn alta_sin_huella_revierte_cliente_historial_y_pago_ante_error() {
        use crate::test_support::*;
        let db = crate::test_support::db();
        let anterior = vigente(&db);
        let antes = counts(&db);
        db.execute_batch("CREATE TRIGGER fallo_pago BEFORE INSERT ON pagos BEGIN SELECT RAISE(ABORT,'fallo simulado'); END;").unwrap();
        assert!(guardar_cliente_sin_duplicar(&db,sin_huella(plan(&db,"Mensual")),|_,_|panic!("SDK")).is_err());
        assert_eq!(counts(&db),antes);
        assert!(db.is_autocommit());
        assert!(registrar_asistencia(&db,anterior).is_ok());
    }
    // Fixture de negocio sin dispositivo; el INSERT real es el de producción.
    pub fn guardar_cliente(conexion: &Connection, datos: DatosCliente) -> Result<i64,String> {
        let tx=conexion.unchecked_transaction().map_err(|e|e.to_string())?;
        let id=insertar_cliente(&tx,datos)?;
        tx.commit().map_err(|e|e.to_string())?;
        Ok(id)
    }
    #[test]
    fn altas_concurrentes_con_misma_huella_solo_generan_un_cobro() {
        let temp=crate::test_support::TempDb::new();let db=temp.connect();crate::database::inicializar_esquema(&db).unwrap();
        let plan=crate::test_support::plan(&db,"Mensual");drop(db);
        let barrier=std::sync::Arc::new(std::sync::Barrier::new(4));
        let threads:Vec<_>=(0..4).map(|_|{let path=temp.path.clone();let b=barrier.clone();std::thread::spawn(move||{
            let db=Connection::open(path).unwrap();db.busy_timeout(std::time::Duration::from_secs(10)).unwrap();b.wait();
            guardar_cliente_sin_duplicar(&db,crate::test_support::cliente(plan),|a,b|Ok(a==b))
        })}).collect();
        assert_eq!(threads.into_iter().map(|t|t.join().unwrap()).filter(Result::is_ok).count(),1);
        assert_eq!(crate::test_support::counts(&temp.connect()),(1,1,1,0));
    }
    #[test]
    fn renovacion_rechaza_fechas_presentes_corruptas() {
        let hoy = chrono::NaiveDate::from_ymd_opt(2026, 11, 6).unwrap();
        for invalida in ["corrupta", "2023-02-29", "2024-13-01", " ", "2024-01-01 basura"] {
            for (inicio, fin, campo) in [
                (invalida, "2027-01-01", "inicio"),
                (invalida, "2020-01-01", "inicio"),
                ("2024-01-01", invalida, "vencimiento"),
            ] {
                let error = periodo_renovacion(hoy, inicio, fin, 1).unwrap_err();
                assert!(error.contains(campo), "{error}");
            }
        }
    }

    #[test]
    fn renovacion_corrupta_no_modifica_cliente_historial_ni_pagos() {
        use crate::test_support::{counts, plan, vigente};
        let db = crate::test_support::db();
        let id = vigente(&db);
        let plan_id = plan(&db, "Mensual");
        for (inicio, fin) in [
            ("corrupta", "2099-01-01"),
            ("corrupta", "2020-01-01"),
            ("2024-01-01", "2023-02-29"),
            ("", "corrupta"),
            ("corrupta", ""),
        ] {
            db.execute("UPDATE clientes SET fecha_inicio_membresia=?1,fecha_fin_membresia=?2 WHERE id=?3",
                params![inicio, fin, id]).unwrap();
            let antes = counts(&db);
            let snapshot = || {
                let mut dump = Vec::new();
                for tabla in ["clientes", "membresias_historial", "pagos"] {
                    let mut stmt = db.prepare(&format!("SELECT * FROM {tabla} ORDER BY id")).unwrap();
                    let columnas = stmt.column_count();
                    let filas = stmt.query_map([], |row| {
                        (0..columnas).map(|i| row.get::<_, rusqlite::types::Value>(i)).collect::<rusqlite::Result<Vec<_>>>()
                    }).unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap();
                    dump.push(filas);
                }
                dump
            };
            let datos_antes = snapshot();
            assert!(renovar_membresia(&db, id, plan_id).unwrap_err().contains("almacenada"));
            assert_eq!(counts(&db), antes);
            assert_eq!(snapshot(), datos_antes);
            assert!(db.is_autocommit());
        }
    }

    #[test]
    fn renovacion_fechas_ausentes_conserva_comportamiento() {
        let hoy = chrono::NaiveDate::from_ymd_opt(2026, 11, 6).unwrap();
        let esperado = chrono::NaiveDate::from_ymd_opt(2026, 12, 6).unwrap();
        assert_eq!(periodo_renovacion(hoy, "", "", 1).unwrap(), (hoy, hoy, esperado));
        let fin = chrono::NaiveDate::from_ymd_opt(2026, 12, 1).unwrap();
        assert_eq!(periodo_renovacion(hoy, "", "2026-12-01", 1).unwrap(),
            (hoy, fin.succ_opt().unwrap(), chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()));
    }

    #[test]
    fn renovacion_calendario_bisiesto_fin_de_mes_y_anio() {
        for (hoy,inicio,fin,meses,esperado) in [
            ("2024-01-01","2023-12-31","2024-01-31",1,"2024-02-29"),
            ("2023-01-01","2022-12-31","2023-01-31",1,"2023-02-28"),
            ("2024-01-01","2024-02-01","2024-02-29",12,"2025-02-28"),
            ("2024-01-01","2023-12-31","2024-03-31",1,"2024-04-30"),
            ("2024-01-01","2023-12-31","2024-12-31",1,"2025-01-31"),
            ("2024-02-29","2023-01-01","2023-12-31",1,"2024-03-29"),
        ] {
            let parse=|s|chrono::NaiveDate::parse_from_str(s,"%Y-%m-%d").unwrap();
            let (inicio_cliente,inicio_hist,finaliza)=periodo_renovacion(parse(hoy),inicio,fin,meses).unwrap();
            assert_eq!(finaliza,parse(esperado));
            if fin>=hoy { assert_eq!(inicio_cliente,parse(inicio)); assert_eq!(inicio_hist,parse(fin).succ_opt().unwrap()); }
            else { assert_eq!(inicio_cliente,parse(hoy));assert_eq!(inicio_hist,parse(hoy)); }
        }
    }
    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;
          CREATE TABLE clientes (
            id INTEGER PRIMARY KEY, nombre TEXT NOT NULL, telefono TEXT,
            telefono_emergencia TEXT, fecha_inicio_membresia TEXT,
            fecha_fin_membresia TEXT, activo INTEGER NOT NULL DEFAULT 1
          );
          CREATE TABLE asistencias (
            id INTEGER PRIMARY KEY, cliente_id INTEGER NOT NULL,
            fecha_hora TEXT NOT NULL, FOREIGN KEY(cliente_id) REFERENCES clientes(id)
          );").unwrap();
        conn
    }
    #[test]
    fn membresia_futura_no_permite_acceso() {
        let conn = db();
        conn.execute("INSERT INTO clientes(id,nombre,fecha_inicio_membresia,fecha_fin_membresia) VALUES(1,'Test',date('now','localtime','+1 day'),date('now','localtime','+30 day'))", []).unwrap();
        assert!(registrar_asistencia(&conn, 1).unwrap_err().contains("pendiente"));
    }
    #[test]
    fn membresia_vigente_registra_solo_una_vez() {
        let conn = db();
        conn.execute("INSERT INTO clientes(id,nombre,fecha_inicio_membresia,fecha_fin_membresia) VALUES(1,'Test',date('now','localtime'),date('now','localtime','+30 day'))", []).unwrap();
        assert!(registrar_asistencia(&conn, 1).is_ok());
        assert!(registrar_asistencia(&conn, 1).unwrap_err().contains("Ya registró"));
    }
    #[test]
    fn membresia_vencida_no_permite_acceso() {
        let conn = db();
        conn.execute("INSERT INTO clientes(id,nombre,fecha_inicio_membresia,fecha_fin_membresia) VALUES(1,'Test',date('now','localtime','-30 day'),date('now','localtime','-1 day'))", []).unwrap();
        assert!(registrar_asistencia(&conn, 1).unwrap_err().contains("vencida"));
    }
    #[test]
    fn edicion_rechaza_datos_invalidos() {
        let conn = db();
        let datos = ActualizacionCliente {
            nombre: "Persona".into(), telefono: "abc".into(), telefono_emergencia: "".into(),
            fecha_inicio_membresia: "2026-10-30".into(), fecha_fin_membresia: "2026-11-30".into(),
        };
        assert!(actualizar_cliente(&conn, 1, datos).is_err());
    }
