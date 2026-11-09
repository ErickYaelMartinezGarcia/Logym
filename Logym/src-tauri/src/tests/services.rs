use super::*;
use crate::test_support::cliente;

#[test]
fn fecha_alta_siempre_procede_del_backend() {
    for falsa in ["1900-01-01", "2099-12-31", "", "fecha manipulada"] {
        let mut datos=cliente(1);datos.fecha_registro=falsa.into();validar_alta(&mut datos).unwrap();
        assert!(datos.fecha_registro.starts_with(&chrono::Local::now().format("%Y-%m-%d").to_string()));
        assert_ne!(datos.fecha_registro,falsa);
    }
}

#[test]
fn alta_semanal_ambas_modalidades_cobro_historico_y_fin_inclusivo() {
    use crate::test_support::*;
    let db=db();
    db.execute("INSERT INTO tipos_membresia(nombre,duracion_meses,duracion_dias,precio_centavos) VALUES('Semanal',1,7,12000)",[]).unwrap();
    let plan_id=plan(&db,"Semanal");
    for (sin_huella,inscripcion) in [(false,true),(true,false)] {
        let mut datos=cliente(plan_id);datos.sin_huella=sin_huella;datos.incluir_inscripcion=inscripcion;
        if sin_huella {datos.template_huella.clear();}
        datos.fecha_inicio_membresia="2026-11-08".into();datos.fecha_fin_membresia="2026-12-08".into();
        validar_alta(&mut datos).unwrap();completar_fin_alta(&mut datos,1,Some(7)).unwrap();
        assert_eq!(datos.fecha_fin_membresia,"2026-11-14");
        let id=crate::repositories::guardar_cliente_sin_duplicar(&db,datos,|_,_|Ok(false)).unwrap();
        let pago:(i64,i64,i64)=db.query_row("SELECT membresia_centavos,inscripcion_centavos,total_centavos FROM pagos WHERE cliente_id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert_eq!(pago,(12000,if inscripcion {15000}else{0},if inscripcion {27000}else{12000}));
    }
    db.execute("UPDATE tipos_membresia SET precio_centavos=99000 WHERE id=?1",[plan_id]).unwrap();
    assert_eq!(db.query_row("SELECT sum(membresia_centavos) FROM pagos",[],|r|r.get::<_,i64>(0)).unwrap(),24000);
}

#[test]
fn asistencia_manual_sin_password_reutiliza_validaciones() {
    use crate::test_support::*;
    let db = crate::test_support::db();
    let id = vigente(&db);
    assert!(!crate::membresias::tiene_administrador(&db).unwrap());
    registrar_asistencia_manual(&db, id).unwrap();
    assert!(registrar_asistencia_manual(&db, id).unwrap_err().contains("Ya registró"));
    for (inicio, fin, activo, error) in [
        ("-30 days", "-1 day", 1, "vencida"),
        ("+1 day", "+30 days", 1, "pendiente"),
        ("-1 day", "+30 days", 0, "desactivado"),
    ] {
        db.execute("UPDATE clientes SET fecha_inicio_membresia=date('now','localtime',?1),fecha_fin_membresia=date('now','localtime',?2),activo=?3 WHERE id=?4", params![inicio,fin,activo,id]).unwrap();
        assert!(registrar_asistencia_manual(&db, id).unwrap_err().contains(error));
    }
    assert_eq!(counts(&db), (1,1,1,1));
}

#[test]
fn alta_sin_huella_es_explicita_y_conserva_validaciones() {
    let mut d = cliente(1);
    d.template_huella.clear();
    assert!(validar_alta(&mut d).is_err());
    d.sin_huella = true;
    validar_alta(&mut d).unwrap();
    d.template_huella = "ab".repeat(400);
    assert!(validar_alta(&mut d).is_err(), "No aceptar datos biométricos contradictorios");
    d.template_huella.clear();
    d.nombre.clear();
    assert!(validar_alta(&mut d).is_err());
    let old: DatosCliente = serde_json::from_str(&format!(r#"{{"nombre":"Anterior","telefono":"","templateHuella":"{}"}}"#, "ab".repeat(400))).unwrap();
    assert!(!old.sin_huella, "IPC existente conserva modalidad biométrica");
}

#[test]
fn busqueda_manual_distingue_homonimos_y_no_expone_inactivos() {
    let db = crate::test_support::db();
    let a = crate::test_support::vigente(&db);
    let b = crate::test_support::vigente(&db);
    db.execute("UPDATE clientes SET nombre='Alex Rivera',telefono='5511111111' WHERE id=?1", [a]).unwrap();
    db.execute("UPDATE clientes SET nombre='Alex Rivera',telefono='5522222222' WHERE id=?1", [b]).unwrap();
    let results = buscar_clientes_checkin_en(&db, "Alex").unwrap();
    assert_eq!(results.len(), 2);
    assert_ne!(results[0].id, results[1].id);
    assert_ne!(results[0].telefono, results[1].telefono);
    assert!(!results[0].estado_membresia.is_empty());
    assert!(!serde_json::to_string(&results).unwrap().contains("template_huella"));
    assert_eq!(buscar_clientes_checkin_en(&db, "5522222222").unwrap()[0].id, b);
    assert!(buscar_clientes_checkin_en(&db, "' OR 1=1 --").unwrap().is_empty());
    assert!(buscar_clientes_checkin_en(&db, " ").unwrap().is_empty());
    db.execute("UPDATE clientes SET activo=0 WHERE id=?1", [a]).unwrap();
    assert_eq!(buscar_clientes_checkin_en(&db, "Alex").unwrap().len(), 1);
}

#[test]
fn alta_rechaza_campos_invalidos_sin_abrir_base_real() {
    let cases: Vec<(&str,String)> = vec![
        ("nombre"," ".into()),("nombre","x".repeat(121)),("nombre","a\0b".into()),
        ("telefono","abc".into()),("telefono","1".repeat(26)),("emergencia","bad".into()),
        ("template","".into()),("template","aa".repeat(399)),("template","aa".repeat(401)),
        ("template","zz".repeat(400)),("template","é".repeat(400)),
        ("foto","data:text/html;base64,PHNjcmlwdD4=".into()),("foto","x".repeat(3_000_001)),
        ("inicio","2024-04-31".into()),("fin","2023-01-01".into()),
    ];
    for (field,value) in cases {
        let mut d=cliente(1);
        match field {"nombre"=>d.nombre=value,"telefono"=>d.telefono=value,"emergencia"=>d.telefono_emergencia=value,
            "template"=>d.template_huella=value,"foto"=>d.foto_base64=value,"registro"=>d.fecha_registro=value,
            "inicio"=>d.fecha_inicio_membresia=value,_=>d.fecha_fin_membresia=value}
        assert!(validar_alta(&mut d).is_err(),"Campo {field}");
    }
}

#[test]
fn alta_admite_limites_unicode_telefonos_opcionales_y_fotos_soportadas() {
    for prefix in ["","data:image/jpeg;base64,","data:image/png;base64,","data:image/webp;base64,"] {
        let mut d=cliente(1);d.nombre=format!(" {} ","ñ".repeat(120));d.telefono=String::new();
        d.telefono_emergencia="+52 (55) 1234-5678".into();d.template_huella="AF".repeat(400);d.foto_base64=prefix.into();
        validar_alta(&mut d).unwrap();assert_eq!(d.nombre.chars().count(),120);
    }
}

#[test]
fn calendario_alta_28_29_30_31_bisiestos_siglos_y_cambios_de_anio() {
    for (inicio,meses,expected) in [
        ("2023-01-31",1,"2023-02-28"),("2024-01-31",1,"2024-02-29"),
        ("2024-02-29",1,"2024-03-29"),("2024-02-29",12,"2025-02-28"),
        ("2024-03-31",1,"2024-04-30"),("2024-04-30",1,"2024-05-30"),
        ("2024-12-31",1,"2025-01-31"),("2024-08-31",6,"2025-02-28"),
        ("1900-01-31",1,"1900-02-28"),("2000-01-31",1,"2000-02-29"),
        ("2024-11-30",2,"2025-01-30"),("2024-06-28",1,"2024-07-28"),
    ] {
        let mut d=cliente(1);d.fecha_inicio_membresia=inicio.into();d.fecha_fin_membresia=String::new();
        completar_fin_alta(&mut d,meses,None).unwrap();assert_eq!(d.fecha_fin_membresia,expected,"{inicio} + {meses}");
    }
}

#[test]
fn calendario_produccion_para_todos_los_planes_en_cada_dia_del_anio() {
    let db=crate::test_support::db();
    for plan in crate::membresias::listar(&db).unwrap() {
        for year in [2023,2024] {
            let mut date=chrono::NaiveDate::from_ymd_opt(year,1,1).unwrap();
            while date.year()==year {
                let mut d=cliente(plan.id);d.fecha_inicio_membresia=date.to_string();
                completar_fin_alta(&mut d,plan.duracion_meses,None).unwrap();
                let fin=chrono::NaiveDate::parse_from_str(&d.fecha_fin_membresia,"%Y-%m-%d").unwrap();
                let target=(date.month0() as i64+plan.duracion_meses) as u32;
                assert_eq!((fin.year(),fin.month0()),(year+(target/12) as i32,target%12));
                assert!(fin>date);assert!(fin.day()<=date.day());
                date=date.succ_opt().unwrap();
            }
        }
    }
}
use chrono::Datelike;

#[test]
fn modelos_ipc_rechazan_tipos_incorrectos_y_no_confian_en_campos_extra() {
    for json in [r#"{}"#,r#"{"nombre":1,"telefono":""}"#,r#"{"nombre":"A","telefono":false}"#,
        r#"{"nombre":"A","telefono":"","tipoMembresiaId":9223372036854775808}"#] {
        assert!(serde_json::from_str::<DatosCliente>(json).is_err());
    }
    let d:DatosCliente=serde_json::from_str(r#"{"nombre":"A","telefono":"","incluirInscripcion":false,"precio":1,"activo":0}"#).unwrap();
    assert!(d.tipo_membresia_id.is_none());assert!(!d.incluir_inscripcion);assert!(d.template_huella.is_empty());
}
