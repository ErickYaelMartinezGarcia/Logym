    use super::*;
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
