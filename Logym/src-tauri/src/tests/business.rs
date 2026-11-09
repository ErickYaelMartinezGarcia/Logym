use crate::{database, membresias as m, repositories as r, test_support::*};
use rusqlite::params;
use std::sync::{Arc, Barrier};

#[test]
fn catalogo_completo_precios_duraciones_e_inscripcion() {
    let db = db();
    let catalogo = m::listar(&db).unwrap();
    assert_eq!(catalogo.len(), 5);
    for (nombre, meses, precio, solo_alta) in [
        ("Mensual",1,35000,false),("Pareja",1,30000,true),("Estudiante",1,28000,false),
        ("Bimestral",2,65000,false),("Semestral",6,180000,false),
    ] {
        let p = catalogo.iter().find(|p| p.nombre == nombre).unwrap();
        assert_eq!((p.duracion_meses,p.precio_centavos,p.solo_alta), (meses,precio,solo_alta));
        assert!(p.activo);
        let id = r::guardar_cliente(&db, cliente(p.id)).unwrap();
        let importe: (i64,i64,i64) = db.query_row("SELECT membresia_centavos,inscripcion_centavos,total_centavos FROM pagos WHERE cliente_id=?1",[id],|row| Ok((row.get(0)?,row.get(1)?,row.get(2)?))).unwrap();
        assert_eq!(importe, (precio,15000,precio+15000));
    }
    assert_eq!(m::inscripcion(&db).unwrap(),15000);
}

#[test]
fn altas_sin_inscripcion_y_renovaciones_no_la_cobran() {
    let db = db();
    for p in m::listar(&db).unwrap() {
        let mut datos = cliente(p.id); datos.incluir_inscripcion = false;
        let id = r::guardar_cliente(&db,datos).unwrap();
        let before = counts(&db);
        let result = r::renovar_membresia(&db,id,p.id);
        if p.solo_alta { assert!(result.is_err()); assert_eq!(counts(&db),before); continue; }
        result.unwrap();
        assert_eq!(db.query_row("SELECT sum(inscripcion_centavos) FROM pagos WHERE cliente_id=?1",[id],|r|r.get::<_,i64>(0)).unwrap(),0);
        assert_eq!(db.query_row("SELECT total_centavos FROM pagos WHERE cliente_id=?1 AND tipo='renovacion'",[id],|r|r.get::<_,i64>(0)).unwrap(),p.precio_centavos);
        let expected = chrono::Local::now().date_naive().checked_add_months(chrono::Months::new(p.duracion_meses as u32)).unwrap().to_string();
        let actual: String = db.query_row("SELECT fecha_fin_membresia FROM clientes WHERE id=?1",[id],|r|r.get(0)).unwrap();
        assert_eq!(actual,expected);
    }
}

#[test]
fn renovar_activa_conserva_dias_pagados_y_acceso() {
    let db = db(); let id = vigente(&db);
    let (inicio,fin): (String,String) = db.query_row("SELECT fecha_inicio_membresia,fecha_fin_membresia FROM clientes WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    let expected = chrono::NaiveDate::parse_from_str(&fin,"%Y-%m-%d").unwrap().checked_add_months(chrono::Months::new(1)).unwrap().to_string();
    r::renovar_membresia(&db,id,plan(&db,"Mensual")).unwrap();
    let actual: (String,String) = db.query_row("SELECT fecha_inicio_membresia,fecha_fin_membresia FROM clientes WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(actual,(inicio,expected),"Una renovación no debe quitar días ya pagados");
    assert!(r::registrar_asistencia(&db,id).is_ok());
}

#[test]
fn renovar_futura_conserva_inicio_y_extiende_vencimiento() {
    let db = db(); let id = vigente(&db);
    db.execute("UPDATE clientes SET fecha_inicio_membresia=date('now','localtime','+5 days'), fecha_fin_membresia=date('now','localtime','+35 days') WHERE id=?1",[id]).unwrap();
    let before: (String,String) = db.query_row("SELECT fecha_inicio_membresia,fecha_fin_membresia FROM clientes WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    r::renovar_membresia(&db,id,plan(&db,"Bimestral")).unwrap();
    let after: (String,String) = db.query_row("SELECT fecha_inicio_membresia,fecha_fin_membresia FROM clientes WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
    let expected = chrono::NaiveDate::parse_from_str(&before.1,"%Y-%m-%d").unwrap().checked_add_months(chrono::Months::new(2)).unwrap().to_string();
    assert_eq!(after,(before.0,expected));
    assert!(r::registrar_asistencia(&db,id).is_err());
}

#[test]
fn precios_nuevos_y_nombres_no_reescriben_pagos_historicos() {
    let db = db(); let p = plan(&db,"Mensual"); let id = r::guardar_cliente(&db,cliente(p)).unwrap();
    db.execute("UPDATE tipos_membresia SET precio_centavos=42000,nombre='Mensual actualizado' WHERE id=?1",[p]).unwrap();
    db.execute("UPDATE configuracion_cobros SET inscripcion_centavos=25000",[]).unwrap();
    r::renovar_membresia(&db,id,p).unwrap();
    let rows: Vec<(String,i64,i64,i64)> = db.prepare("SELECT concepto_membresia,membresia_centavos,inscripcion_centavos,total_centavos FROM pagos ORDER BY id").unwrap()
        .query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap().map(Result::unwrap).collect();
    assert_eq!(rows,vec![("Mensual".into(),35000,15000,50000),("Mensual actualizado".into(),42000,0,42000)]);
    assert_eq!(m::ingresos(&db,"dia").unwrap().total_centavos,92000);
}

#[test]
fn asistencias_estados_limites_inclusivos_y_clientes_ausentes() {
    let db = db();
    for (start,end,activo,permitir) in [(0,0,1,true),(-30,-1,1,false),(1,30,1,false),(-1,1,0,false),(-10,10,1,true)] {
        let id = vigente(&db);
        db.execute("UPDATE clientes SET fecha_inicio_membresia=date('now','localtime',?1),fecha_fin_membresia=date('now','localtime',?2),activo=?3 WHERE id=?4",params![format!("{start:+} days"),format!("{end:+} days"),activo,id]).unwrap();
        assert_eq!(r::registrar_asistencia(&db,id).is_ok(),permitir);
        assert!(r::registrar_asistencia(&db,id).is_err());
    }
    assert!(r::registrar_asistencia(&db,i64::MAX).is_err());
    assert!(r::registrar_asistencia(&db,-1).is_err());
    assert_eq!(counts(&db).3,2);
}

#[test]
fn asistencia_ayer_235959_no_impide_hoy_y_hoy_000000_si() {
    let db = db(); let id = vigente(&db);
    db.execute("INSERT INTO asistencias(cliente_id,fecha_hora) VALUES(?1,date('now','localtime','-1 day') || ' 23:59:59')",[id]).unwrap();
    assert!(r::registrar_asistencia(&db,id).is_ok());
    db.execute("UPDATE asistencias SET fecha_hora=date('now','localtime') || ' 00:00:00' WHERE id=(SELECT max(id) FROM asistencias)",[]).unwrap();
    assert!(r::registrar_asistencia(&db,id).is_err());
    assert_eq!(counts(&db).3,2);
}

#[test]
fn estados_listados_activa_por_vencer_pendiente_vencida_sin_fecha() {
    let db = db();
    for (start,end,expected) in [("-5 days",Some("+20 days"),"activa"),("-5 days",Some("+7 days"),"por_vencer"),("+1 day",Some("+30 days"),"pendiente"),("-30 days",Some("-1 day"),"vencida"),("-1 day",None,"sin_fecha")] {
        let id = vigente(&db);
        db.execute("UPDATE clientes SET fecha_inicio_membresia=date('now','localtime',?1),fecha_fin_membresia=COALESCE(date('now','localtime',?2),'') WHERE id=?3",params![start,end,id]).unwrap();
        let rows = r::listar_clientes_membresia(&db).unwrap();
        assert_eq!(rows.iter().find(|r|r.id==id).unwrap().estado_membresia,expected);
    }
    let id=vigente(&db);r::establecer_cliente_activo(&db,id,false).unwrap();
    assert!(!r::listar_clientes_membresia(&db).unwrap().iter().find(|c|c.id==id).unwrap().activo);
    assert!(r::registrar_asistencia(&db,id).is_err());r::establecer_cliente_activo(&db,id,true).unwrap();assert!(r::registrar_asistencia(&db,id).is_ok());
}

#[test]
fn alta_fallida_revierte_cliente_historial_y_pago() {
    for tabla in ["pagos","membresias_historial"] {
        let db = db();
        db.execute_batch(&format!("CREATE TRIGGER fallo BEFORE INSERT ON {tabla} BEGIN SELECT RAISE(ABORT,'fallo inyectado'); END;")).unwrap();
        assert!(r::guardar_cliente(&db,cliente(plan(&db,"Mensual"))).is_err());assert_eq!(counts(&db),(0,0,0,0));
    }
}

#[test]
fn renovacion_fallida_revierte_fechas_historial_y_pago() {
    for tabla in ["pagos","membresias_historial"] {
        let db = db(); let id = vigente(&db);let before=counts(&db);
        let fechas: (String,String)=db.query_row("SELECT fecha_inicio_membresia,fecha_fin_membresia FROM clientes WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        db.execute_batch(&format!("CREATE TRIGGER fallo BEFORE INSERT ON {tabla} BEGIN SELECT RAISE(ABORT,'fallo inyectado'); END;")).unwrap();
        assert!(r::renovar_membresia(&db,id,plan(&db,"Mensual")).is_err());assert_eq!(counts(&db),before);
        assert_eq!(db.query_row("SELECT fecha_inicio_membresia,fecha_fin_membresia FROM clientes WHERE id=?1",[id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).unwrap(),fechas);
    }
}

#[test]
fn planes_inactivos_inexistentes_y_clientes_inactivos_no_dejan_registros() {
    let db=db();let id=vigente(&db);r::establecer_cliente_activo(&db,id,false).unwrap();let before=counts(&db);
    assert!(r::renovar_membresia(&db,id,plan(&db,"Mensual")).is_err());
    assert!(r::renovar_membresia(&db,9999,plan(&db,"Mensual")).is_err());
    for p in [None,Some(-1),Some(i64::MAX)] {let mut datos=cliente(1);datos.tipo_membresia_id=p;assert!(r::guardar_cliente(&db,datos).is_err());}
    let p=plan(&db,"Estudiante");db.execute("UPDATE tipos_membresia SET activo=0 WHERE id=?1",[p]).unwrap();
    assert!(r::guardar_cliente(&db,cliente(p)).is_err());assert!(r::renovar_membresia(&db,id,p).is_err());assert_eq!(counts(&db),before);
}

#[test]
fn eliminacion_cascada_y_restriccion_del_plan_pagado() {
    let mut db=db();let id=vigente(&db);r::registrar_asistencia(&db,id).unwrap();
    assert!(db.execute("DELETE FROM tipos_membresia WHERE id=?1",[plan(&db,"Mensual")]).is_err());
    r::eliminar_cliente_definitivamente(&mut db,id).unwrap();assert_eq!(counts(&db),(0,0,0,0));
    assert!(r::eliminar_cliente_definitivamente(&mut db,id).is_err());
}

#[test]
fn concurrencia_asistencia_mismo_cliente_solo_una_escritura() {
    let fixture=TempDb::new();let db=fixture.connect();database::inicializar_esquema(&db).unwrap();let id=vigente(&db);drop(db);
    let barrier=Arc::new(Barrier::new(8));
    let handles:Vec<_>=(0..8).map(|_|{let path=fixture.path.clone();let b=barrier.clone();std::thread::spawn(move||{let db=rusqlite::Connection::open(path).unwrap();db.busy_timeout(std::time::Duration::from_secs(10)).unwrap();db.execute_batch("PRAGMA foreign_keys=ON").unwrap();b.wait();r::registrar_asistencia(&db,id)})}).collect();
    let results:Vec<_>=handles.into_iter().map(|h|h.join().unwrap()).collect();assert_eq!(results.iter().filter(|r|r.is_ok()).count(),1,"{results:?}");
    assert_eq!(counts(&fixture.connect()).3,1);
}

#[test]
fn concurrencia_renovacion_no_deja_pagos_parciales() {
    let fixture=TempDb::new();let db=fixture.connect();database::inicializar_esquema(&db).unwrap();let id=vigente(&db);let p=plan(&db,"Mensual");drop(db);
    let barrier=Arc::new(Barrier::new(4));let handles:Vec<_>=(0..4).map(|_|{let path=fixture.path.clone();let b=barrier.clone();std::thread::spawn(move||{let db=rusqlite::Connection::open(path).unwrap();db.busy_timeout(std::time::Duration::from_secs(10)).unwrap();b.wait();r::renovar_membresia(&db,id,p)})}).collect();
    let successes=handles.into_iter().map(|h|h.join().unwrap()).filter(Result::is_ok).count() as i64;
    assert_eq!(counts(&fixture.connect()),(1,1+successes,1+successes,0));
    assert!(successes>0);
    let db=fixture.connect();let mut expected=chrono::Local::now().date_naive().checked_add_signed(chrono::Duration::days(30)).unwrap();
    for _ in 0..successes { expected=expected.checked_add_months(chrono::Months::new(1)).unwrap(); }
    assert_eq!(db.query_row("SELECT fecha_fin_membresia FROM clientes WHERE id=?1",[id],|r|r.get::<_,String>(0)).unwrap(),expected.to_string());
}

#[test]
fn edicion_rechaza_extremos_y_conserva_huella() {
    let db=db();let id=vigente(&db);
    let datos=||crate::models::ActualizacionCliente{nombre:" Editado ".into(),telefono:"5512345678".into(),telefono_emergencia:String::new(),fecha_inicio_membresia:"2024-02-29".into(),fecha_fin_membresia:"2025-02-28".into()};
    for (field,value) in [(0,"".into()),(0,"x".repeat(121)),(0,"a\nb".into()),(1,"a".into()),(1,"1".repeat(26)),(2,"2023-02-29".into()),(3,"2020-01-01".into())] {
        let mut d=datos();match field{0=>d.nombre=value,1=>d.telefono=value,2=>d.fecha_inicio_membresia=value,_=>d.fecha_fin_membresia=value};assert!(r::actualizar_cliente(&db,id,d).is_err());
    }
    r::actualizar_cliente(&db,id,datos()).unwrap();assert_eq!(db.query_row("SELECT template_huella FROM clientes WHERE id=?1",[id],|r|r.get::<_,String>(0)).unwrap(),"ab".repeat(400));
    assert!(r::actualizar_cliente(&db,-1,datos()).is_err());
}

#[test]
fn entradas_sql_se_persisten_como_datos_y_no_ejecutan_sentencias() {
    let db=db();let mut datos=cliente(plan(&db,"Mensual"));datos.nombre="O'Brien'); DROP TABLE pagos; --".into();let nombre=datos.nombre.clone();let id=r::guardar_cliente(&db,datos).unwrap();
    assert_eq!(db.query_row("SELECT nombre FROM clientes WHERE id=?1",[id],|r|r.get::<_,String>(0)).unwrap(),nombre);assert_eq!(counts(&db),(1,1,1,0));
    assert!(m::ingresos(&db,"dia';DROP TABLE pagos;--").is_err());assert_eq!(counts(&db).1,1);
}

#[test]
fn autenticacion_hash_salteado_y_configuracion_unica() {
    let a=db();let b=db();let password="Clave-sintetica-123";
    assert!(!m::tiene_administrador(&a).unwrap());assert!(m::autenticar(&a,password).is_err());
    for bad in ["".to_string(),"x".repeat(11),"x".repeat(129)]{assert!(m::configurar_administrador(&a,&bad).is_err());}
    m::configurar_administrador(&a,password).unwrap();m::configurar_administrador(&b,password).unwrap();
    let hash=|db:&rusqlite::Connection|db.query_row("SELECT password_hash FROM administrador",[],|r|r.get::<_,String>(0)).unwrap();
    assert!(hash(&a).starts_with("$argon2id$"));assert_ne!(hash(&a),hash(&b));assert!(!hash(&a).contains(password));
    assert!(m::configurar_administrador(&a,"Otra-clave-1234").is_err());assert!(m::autenticar(&a,"incorrecta").is_err());m::autenticar(&a,password).unwrap();
    let before=m::inscripcion(&a).unwrap();assert!(m::actualizar_inscripcion(&a,"' OR 1=1 --",0).is_err());assert_eq!(m::inscripcion(&a).unwrap(),before);
}

#[test]
fn administrador_valida_planes_importes_y_limites() {
    let db=db();let pass="Prueba-admin-123";m::configurar_administrador(&db,pass).unwrap();
    for (nombre,meses,precio) in [("",1,1),("a\nb",1,1),("Plan",0,1),("Plan",37,1),("Plan",1,-1),("Plan",1,100_000_001)] {assert!(m::actualizar_plan(&db,pass,None,nombre,meses,precio,true,None).is_err());}
    assert!(m::actualizar_plan(&db,pass,Some(i64::MAX),"Ausente",1,1,true,None).is_err());
    for precio in [-1,i64::MAX] {assert!(m::actualizar_inscripcion(&db,pass,precio).is_err());}
    m::actualizar_plan(&db,pass,None,"Plan prueba",36,100_000_000,true,None).unwrap();m::actualizar_inscripcion(&db,pass,0).unwrap();assert_eq!(m::inscripcion(&db).unwrap(),0);
    let id=plan(&db,"Plan prueba");m::actualizar_plan(&db,pass,Some(id),"Plan prueba",1,0,false,None).unwrap();assert!(!m::listar(&db).unwrap().iter().any(|p|p.id==id));assert_eq!(m::listar_admin(&db,pass).unwrap().len(),6);
}

#[test]
fn reportes_calendario_bisiestos_rangos_inclusivos_y_fechas_invalidas() {
    let db=db();let id=vigente(&db);db.execute("UPDATE pagos SET fecha_pago='2024-02-29 23:59:59' WHERE cliente_id=?1",[id]).unwrap();
    for (start,end) in [("2024-02-29","2024-02-29"),("2024-02-28","2024-03-01")] {assert_eq!(m::ingresos_rango(&db,start,end).unwrap().total_centavos,50000);}
    assert_eq!(m::ingresos_rango(&db,"2024-03-01","2024-03-31").unwrap().numero_pagos,0);
    for bad in ["2023-02-29","1900-02-29","2024-04-31","0000-01-01","2024-13-01","2024-01-00","2024-1-01","2024-01-01' OR 1=1 --","ééééé"]{assert!(m::ingresos_rango(&db,bad,"9999-12-31").is_err(),"{bad}");}
    assert!(m::ingresos_rango(&db,"2025-01-01","2024-01-01").is_err());assert!(m::ingresos_rango(&db,"2000-02-29","2000-02-29").is_ok());
    for period in ["dia","semana","mes","anual"]{assert!(m::ingresos(&db,period).is_ok());}
    for period in ["dia","semana","mes"]{assert!(m::asistencias_periodo(&db,period).is_ok());assert!(m::asistencias_diarias(&db,period).is_ok());}
    assert!(m::asistencias_periodo(&db,"anual").is_err());
}

#[test]
fn huellas_sinteticas_identicas_caracterizan_ausencia_de_unicidad() {
    let db=db();let p=plan(&db,"Mensual");r::guardar_cliente(&db,cliente(p)).unwrap();r::guardar_cliente(&db,cliente(p)).unwrap();
    // Caracterización, no una afirmación de comparación biométrica real.
    assert_eq!(db.query_row("SELECT count(*) FROM clientes WHERE template_huella=?1",["ab".repeat(400)],|r|r.get::<_,i64>(0)).unwrap(),2);
}
