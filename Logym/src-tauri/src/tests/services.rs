use super::*;
use crate::test_support::cliente;

#[test]
fn alta_rechaza_campos_invalidos_sin_abrir_base_real() {
    let cases: Vec<(&str,String)> = vec![
        ("nombre"," ".into()),("nombre","x".repeat(121)),("nombre","a\0b".into()),
        ("telefono","abc".into()),("telefono","1".repeat(26)),("emergencia","bad".into()),
        ("template","".into()),("template","aa".repeat(399)),("template","aa".repeat(401)),
        ("template","zz".repeat(400)),("template","é".repeat(400)),
        ("foto","data:text/html;base64,PHNjcmlwdD4=".into()),("foto","x".repeat(3_000_001)),
        ("registro","2023-02-29".into()),("inicio","2024-04-31".into()),("fin","2023-01-01".into()),
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
        completar_fin_alta(&mut d,meses).unwrap();assert_eq!(d.fecha_fin_membresia,expected,"{inicio} + {meses}");
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
                completar_fin_alta(&mut d,plan.duracion_meses).unwrap();
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
