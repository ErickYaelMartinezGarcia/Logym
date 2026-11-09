    use super::*;
    #[test]
    fn categorias_financieras_con_duracion_persistida_sin_sumar_semanales_dos_veces() {
        use crate::test_support::*;
        let db=db();let id=vigente(&db);
        db.execute("INSERT INTO tipos_membresia(nombre,duracion_meses,duracion_dias,precio_centavos) VALUES('Nombre arbitrario',1,7,12000)",[]).unwrap();let p=plan(&db,"Nombre arbitrario");
        let mut datos=cliente(p);datos.incluir_inscripcion=false;crate::repositories::guardar_cliente(&db,datos).unwrap();
        crate::repositories::renovar_membresia_operacion(&db,id,p,&"c".repeat(32)).unwrap();
        crate::visitas::registrar(&db,&"d".repeat(32),"").unwrap();
        // Un pago histórico sin duración conserva tipo e importe, nunca se
        // clasifica a partir del nombre o de la duración actual del catálogo.
        db.execute("UPDATE pagos SET duracion_dias=NULL,duracion_meses=NULL WHERE cliente_id=?1 AND tipo='alta'",[id]).unwrap();
        db.execute("UPDATE tipos_membresia SET nombre='Mensual renombrado',duracion_dias=30,precio_centavos=99000 WHERE id=?1",[p]).unwrap();
        let hoy=chrono::Local::now().format("%Y-%m-%d").to_string();
        for (cat,importe,n) in [("todos",79000,4),("altas",62000,2),("renovaciones",12000,1),("visitas",5000,1),("semanales",24000,2)] {
            let r=ingresos_rango_categoria(&db,&hoy,&hoy,cat).unwrap();
            assert_eq!((r.total_centavos,r.numero_pagos,r.total_periodo_centavos),(importe,n,79000));
            assert_eq!(r.serie.iter().map(|p|p.total_centavos).sum::<i64>(),importe);
            assert_eq!(r.movimientos.iter().map(|m|m.total_centavos).sum::<i64>(),importe);
        }
        let vacio=ingresos_rango_categoria(&db,"2000-01-01","2000-01-01","todos").unwrap();assert_eq!(vacio.total_centavos,0);
        assert!(ingresos_rango_categoria(&db,&hoy,&hoy,"inventada").is_err());
    }

    #[test]
    fn siete_dias_locales_completos_con_ceros_y_visitas_sin_duplicados() {
        use crate::test_support::*;
        let db=db();let id=vigente(&db);
        db.execute("INSERT INTO asistencias(cliente_id,fecha_hora) VALUES(?1,datetime('now','localtime','-6 days'))",[id]).unwrap();
        db.execute("INSERT INTO asistencias(cliente_id,fecha_hora) VALUES(?1,datetime('now','localtime','-7 days'))",[id]).unwrap();
        crate::visitas::registrar(&db,&"e".repeat(32),"").unwrap();
        let datos=asistencias_diarias(&db,"semana").unwrap();assert_eq!(datos.len(),7);
        assert_eq!(datos.iter().map(|d|d.total).collect::<Vec<_>>(),vec![1,0,0,0,0,0,1]);
        let hoy=chrono::Local::now().date_naive();
        for (i,d) in datos.iter().enumerate(){assert_eq!(d.fecha,(hoy-chrono::Duration::days(6-i as i64)).to_string());}
        assert_eq!(asistencias_periodo(&db,"semana").unwrap(),2);
    }
    #[test]
    fn historical_revenue_ignores_catalog_price_changes() {
        let db = crate::test_support::db();
        crate::test_support::vigente(&db);
        db.execute("UPDATE tipos_membresia SET precio_centavos=40000 WHERE nombre='Mensual'",[]).unwrap();
        let reporte = ingresos(&db,"mes").unwrap();
        assert_eq!(reporte.total_centavos,50000);
        assert_eq!(reporte.membresias_centavos,35000);
        assert_eq!(reporte.inscripciones_centavos,15000);
    }
    #[test]
    fn rejects_invalid_period() {
        let db = Connection::open_in_memory().unwrap();
        assert!(ingresos(&db,"invalido").is_err());
    }
#[test]
fn administrar_plan_en_dias_y_validar_duracion_sin_cambiar_mensuales() {
    use crate::test_support::*;
    let db=db();let password="Password-sintetico-2026";
    super::configurar_administrador(&db,password).unwrap();
    super::actualizar_plan(&db,password,None,"Semanal",1,12000,true,Some(7)).unwrap();
    let id=plan(&db,"Semanal");
    let lista=super::listar(&db).unwrap();
    assert_eq!(lista.iter().find(|p|p.id==id).unwrap().duracion_dias,Some(7));
    assert!(lista.iter().filter(|p|p.id!=id).all(|p|p.duracion_dias.is_none()));
    for (meses,dias) in [(1,Some(0)),(1,Some(-1)),(1,Some(3661)),(2,Some(7))] {
        assert!(super::actualizar_plan(&db,password,Some(id),"Semanal",meses,12000,true,dias).is_err());
    }
    super::actualizar_plan(&db,password,Some(id),"Semanal",1,15000,false,Some(7)).unwrap();
    assert!(!super::listar(&db).unwrap().iter().any(|p|p.id==id));
    assert_eq!(super::listar_admin(&db,password).unwrap().iter().find(|p|p.id==id).unwrap().precio_centavos,15000);
    super::actualizar_plan(&db,password,Some(id),"Mensual configurable",2,15000,true,None).unwrap();
    assert_eq!(super::listar(&db).unwrap().iter().find(|p|p.id==id).unwrap().duracion_dias,None);
}
