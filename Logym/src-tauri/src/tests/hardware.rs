use super::*;

fn equipo() -> Equipo {
    Equipo { uuid: [0x6d,0x23,0x5b,0x81,0xa2,0x4c,0x47,0x12,0x89,0x65,0x21,0xea,0x73,0x98,0x3c,0x50],
        fabricante: "Synthetic Vendor".into(), modelo: "Synthetic Board X2".into(), serie: "SYNTH-BOARD-4829".into() }
}

#[test]
fn hp_y_fabricantes_validos_conservan_normalizacion() {
    for fabricante in ["HP", "Dell", "Lenovo", "ASUS", "Hewlett-Packard"] {
        let mut original = equipo(); original.fabricante = fabricante.into();
        let esperado = identidad(original).unwrap();
        let mut variante = equipo(); variante.fabricante = format!(" \t{} \r\n", fabricante.to_ascii_lowercase());
        assert_eq!(identidad(variante).unwrap(), esperado, "{fabricante}");
    }
}

#[test]
fn hp_no_amplia_excepcion_a_genericos_ni_otros_fabricantes() {
    for fabricante in ["", " ", "H", "AB", "AA", "NA", "N/A", "OEM", "Unknown",
        "Default string", "To be filled by O.E.M.", "System Manufacturer", "000000", "AAAAAA",
        "H-P", "H P", "HP!", "HP\0", "HＰ", "\u{a0}HP"] {
        let mut e = equipo(); e.fabricante = fabricante.into();
        assert!(identidad(e).is_err(), "{fabricante:?}");
    }
}

#[test]
fn hp_no_relaja_validacion_producto_serie_ni_uuid() {
    for valor in ["", "HP", "AB", "AAA", "N/A", "Unknown", "Default string", "To be filled by O.E.M."] {
        let mut e = equipo(); e.fabricante = "HP".into(); e.modelo = valor.into();
        assert!(identidad(e).is_err(), "producto {valor:?}");
        let mut e = equipo(); e.fabricante = "HP".into(); e.serie = valor.into();
        assert!(identidad(e).is_err(), "serie {valor:?}");
    }
    let mut corta = equipo(); corta.fabricante = "HP".into(); corta.serie = "A1B".into();
    assert!(identidad(corta).unwrap_err().contains("serie de placa insuficiente"));
    for uuid in [[0;16], [255;16], [17;16],
        [3,0,2,0,4,0,5,0,0,6,0,7,0,8,0,9],
        [0,2,0,3,0,4,0,5,0,6,0,7,0,8,0,9],
        [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15]] {
        let mut e = equipo(); e.fabricante = "HP".into(); e.uuid = uuid;
        assert!(identidad(e).unwrap_err().contains("UUID SMBIOS"));
    }
}

#[test]
fn hp_coincide_en_smbios_y_cim_sin_cambiar_identidad() {
    let mut e = equipo(); e.fabricante = "HP".into();
    let esperado = identidad(e).unwrap();
    let mut raw = tabla("SSD", "Windows");
    let fabricante = b"Synthetic Vendor\0";
    let inicio = raw.windows(fabricante.len()).position(|v| v == fabricante).unwrap();
    raw.splice(inicio..inicio + fabricante.len(), b"HP\0".iter().copied());
    let longitud = (raw.len() - 8) as u32; raw[4..8].copy_from_slice(&longitud.to_le_bytes());
    let mut lecturas = 0;
    assert_eq!(solicitud_con(|| { lecturas += 1; Ok(raw.clone()) }).unwrap(), esperado);
    assert_eq!(lecturas, 2);
    let mut cim: serde_json::Value = serde_json::from_slice(&cim_fixture()).unwrap();
    for muestra in cim.as_array_mut().unwrap() { muestra["placas"][0]["fabricante"] = " hp ".into(); }
    let json = serde_json::to_vec(&cim).unwrap();
    assert_eq!(interpretar_cim(&json).unwrap(), esperado);
    assert_eq!(resolver(|| Err("SMBIOS no disponible".into()), || interpretar_cim(&json)).unwrap(), esperado);
}

fn tabla(disco: &str, windows: &str) -> Vec<u8> {
    let mut sys = vec![0u8;25]; sys[0]=1; sys[1]=25;
    let mut uuid=equipo().uuid;uuid[..4].reverse();uuid[4..6].reverse();uuid[6..8].reverse();sys[8..24].copy_from_slice(&uuid);
    sys.extend_from_slice(&[0,0]);
    let mut placa=vec![0u8;15];placa[0]=2;placa[1]=15;placa[4]=1;placa[5]=2;placa[7]=3;placa[13]=10;
    placa.extend_from_slice(b"Synthetic Vendor\0Synthetic Board X2\0SYNTH-BOARD-4829\0\0");sys.extend(placa);
    // Registro OEM irrelevante que simula datos variables de disco/SO.
    sys.extend_from_slice(&[128,4,3,0]);sys.extend_from_slice(format!("{disco}/{windows}\0\0").as_bytes());sys.extend_from_slice(&[127,4,4,0,0,0]);
    let mut raw=vec![0,3,2,0];raw.extend_from_slice(&(sys.len() as u32).to_le_bytes());raw.extend(sys);raw
}

#[test]
fn hardware_normaliza_y_parsea_uuid_y_placa_sin_ambiguedad() {
    let original=identidad(equipo()).unwrap();
    let mut variacion=equipo();variacion.fabricante="  synthetic   vendor ".into();variacion.serie=" synth-board-4829 ".into();
    assert_eq!(identidad(variacion).unwrap(),original);
    assert_eq!(identidad(interpretar(&tabla("SSD-A","Windows-A")).unwrap()).unwrap(),original);
    let mut otro=equipo();otro.serie.push('X');assert_ne!(identidad(otro).unwrap(),original);
    let mut otro=equipo();otro.uuid[15]^=1;assert_ne!(identidad(otro).unwrap(),original);
}

#[test]
fn hardware_generico_ausente_o_corrupto_no_genera_identidad() {
    for uuid in [[0;16],[255;16],[17;16],[3,0,2,0,4,0,5,0,0,6,0,7,0,8,0,9]] {
        let mut e=equipo();e.uuid=uuid;assert!(identidad(e).is_err());
    }
    for serie in ["", "To be filled by O.E.M.","Default string","0000000000","123456789", "N/A", "Unknown", "€1234"] {
        let mut e=equipo();e.serie=serie.into();assert!(identidad(e).is_err(),"{serie}");
    }
    let buena=tabla("SSD-A","Windows-A");
    for len in 0..buena.len() { let mut truncada=buena[..len].to_vec();if len>=8 { truncada[4..8].copy_from_slice(&((len-8) as u32).to_le_bytes()); } assert!(interpretar(&truncada).is_err(), "longitud {len}"); }
    let mut vieja=buena.clone();vieja[1]=2;vieja[2]=5;assert!(interpretar(&vieja).is_err());
    let mut indice=buena.clone();indice[8+27+7]=9;assert!(interpretar(&indice).is_err());
    let mut doble=buena.clone();let fin=doble.len()-6;doble.splice(fin..fin,buena[8..35].iter().copied());let tam=(doble.len()-8) as u32;doble[4..8].copy_from_slice(&tam.to_le_bytes());assert!(interpretar(&doble).is_err());
}

fn licencia() -> &'static str { include_str!("fixtures/licencia-hardware-sintetica.txt").trim() }
fn solicitud_simulada(disco: &str, windows: &str) -> String { identidad(interpretar(&tabla(disco,windows)).unwrap()).unwrap() }

#[test]
fn activacion_hardware_firma_real_y_reapertura() {
    let solicitud=solicitud_simulada("SSD-A","Windows-A");
    let carpeta=std::env::temp_dir().join(format!("logym-license-hw-{}",super::super::hex(&{let mut b=[0u8;16];getrandom::getrandom(&mut b).unwrap();b})));
    std::fs::create_dir(&carpeta).unwrap();let archivo=carpeta.join("license.txt");
    super::super::activar_archivo(&archivo,&solicitud,licencia()).unwrap();
    super::super::validar_firma(&solicitud,&std::fs::read_to_string(&archivo).unwrap()).unwrap();
    // Un rechazo no modifica ni borra la licencia guardada ni los archivos SQLCipher.
    let base=carpeta.join("gym.db");let clave=carpeta.join("gym.key.dpapi");
    std::fs::write(&base,b"synthetic encrypted bytes").unwrap();std::fs::write(&clave,b"synthetic protected key").unwrap();
    assert!(super::super::activar_archivo(&archivo,&solicitud,"LOGYM1-obsoleta:00").is_err());
    assert_eq!(std::fs::read_to_string(&archivo).unwrap(),licencia());
    assert_eq!(std::fs::read(&base).unwrap(),b"synthetic encrypted bytes");assert_eq!(std::fs::read(&clave).unwrap(),b"synthetic protected key");
    std::fs::remove_dir_all(carpeta).unwrap();
}

#[test]
fn cambio_ssd_conserva_licencia_firmada() {
    let antes=solicitud_simulada("SSD-A","Windows-A");let despues=solicitud_simulada("SSD-B","Windows-A");
    assert_eq!(antes,despues);super::super::validar_firma(&despues,licencia()).unwrap();
}

#[test]
fn reinstalacion_windows_sin_identidad_dpapi_conserva_licencia() {
    let antes=solicitud_simulada("SSD-A","Windows-A");let despues=solicitud_simulada("SSD-A","Windows-B-sin-installation.dpapi");
    assert_eq!(antes,despues);super::super::validar_firma(&despues,licencia()).unwrap();
}

#[test]
fn copiar_licencia_y_base_a_otra_pc_es_rechazado() {
    let mut otra=equipo();otra.uuid[15]^=1;otra.serie="SYNTH-OTHER-9835".into();
    let otra=identidad(otra).unwrap();assert!(super::super::validar_firma(&otra,licencia()).unwrap_err().contains("otra computadora"));
    assert!(super::super::validar_firma(&solicitud_simulada("SSD-A","Windows-A"),include_str!("fixtures/licencia-sintetica.txt")).is_err());
}

#[test]
fn lectura_inconsistente_o_fallida_no_usa_cache_ni_fallback() {
    let mut primera=true;
    let resultado=solicitud_con(|| { let mut raw=tabla("SSD-A","Windows-A");if !primera { raw[8+23]^=1; } primera=false;Ok(raw) });
    assert!(resultado.unwrap_err().contains("inconsistentes"));
    assert!(solicitud_con(|| Err("SMBIOS inaccesible".into())).is_err());
    assert!(solicitud_con(|| Ok(Vec::new())).is_err());
}

fn cim_fixture() -> Vec<u8> {
    let lectura=serde_json::json!({"sistemas":[{"uuid":"6D235B81-A24C-4712-8965-21EA73983C50"}],"placas":[{"fabricante":"  synthetic   vendor ","modelo":"Synthetic Board X2","serie":"synth-board-4829","principal":true}]});
    serde_json::to_vec(&vec![lectura.clone(),lectura]).unwrap()
}

#[test]
fn identidad_canonica_anterior_y_equivalencia_cim_smbios_se_conservan() {
    let golden="LOGYM2-HW-85c34de2a54996603128c8467d4cf69b5cf73b76b892f09414934233c983790f";
    assert_eq!(solicitud_simulada("SSD","Windows"),golden);
    assert_eq!(interpretar_cim(&cim_fixture()).unwrap(),golden);
    super::super::validar_firma(golden,licencia()).unwrap();
    assert_eq!(resolver(||Ok(golden.into()),||panic!("CIM no debe sustituir un SMBIOS valido")).unwrap(),golden);
}

#[test]
fn placa_oem_corta_o_tipo_no_especificado_no_cambia_identidad() {
    let base=tabla("SSD","Windows");let board=8+27;
    for length in [8,9,10,14,15] {
        let mut raw=base.clone();raw[board+13]=1;
        raw.drain(board+length..board+15);raw[board+1]=length as u8;
        let tam=(raw.len()-8) as u32;raw[4..8].copy_from_slice(&tam.to_le_bytes());
        assert_eq!(identidad(interpretar(&raw).unwrap()).unwrap(),identidad(equipo()).unwrap());
    }
}

#[test]
fn cim_rechaza_genericos_ambiguedad_y_lecturas_inconsistentes() {
    let original:serde_json::Value=serde_json::from_slice(&cim_fixture()).unwrap();
    for (field,value) in [("uuid","00000000-0000-0000-0000-000000000000"),("uuid","zz235b81-a24c-4712-8965-21ea73983c50"),("uuid","{6d235b81-a24c-4712-8965-21ea73983c50}"),("serie","To be filled by O.E.M."),("serie","")] {
        let mut data=original.clone();for i in 0..2 { if field=="uuid" {data[i]["sistemas"][0][field]=value.into()} else {data[i]["placas"][0][field]=value.into()} }assert!(interpretar_cim(&serde_json::to_vec(&data).unwrap()).is_err());
    }
    let mut data=original.clone();data[1]["placas"][0]["serie"]="OTHER-SYNTHETIC".into();assert!(interpretar_cim(&serde_json::to_vec(&data).unwrap()).is_err());
    let mut data=original.clone();data[0]["sistemas"]=serde_json::json!([]);assert!(interpretar_cim(&serde_json::to_vec(&data).unwrap()).is_err());
    let mut data=original.clone();let board=data[0]["placas"][0].clone();data[0]["placas"].as_array_mut().unwrap().push(board);assert!(interpretar_cim(&serde_json::to_vec(&data).unwrap()).is_err());
    assert!(resolver(||Err("Identificadores inconsistentes".into()),||panic!("No ocultar incoherencia")).is_err());
    assert!(resolver(||Err("SMBIOS".into()),||Err("CIM".into())).is_err());
}

#[test]
fn activar_por_cim_y_verificar_tras_reinicio_o_cambio_de_metodo() {
    let dir=crate::test_support::TempDb::new();let file=dir.dir.join("license.txt");
    let primera=resolver(||Err("SMBIOS OEM incompatible".into()),||interpretar_cim(&cim_fixture())).unwrap();
    super::super::activar_archivo(&file,&primera,licencia()).unwrap();
    // Estado nuevo: no se reutiliza la solicitud ni un archivo de identidad.
    let siguiente=resolver(||Err("SMBIOS OEM incompatible".into()),||interpretar_cim(&cim_fixture())).unwrap();
    super::super::validar_firma(&siguiente,&std::fs::read_to_string(&file).unwrap()).unwrap();
    super::super::validar_firma(&solicitud_simulada("SSD nuevo","Windows nuevo"),&std::fs::read_to_string(&file).unwrap()).unwrap();
    let mut otra:serde_json::Value=serde_json::from_slice(&cim_fixture()).unwrap();for i in 0..2 {otra[i]["sistemas"][0]["uuid"]="6D235B81-A24C-4712-8965-21EA73983C51".into();}
    assert!(super::super::validar_firma(&interpretar_cim(&serde_json::to_vec(&otra).unwrap()).unwrap(),licencia()).is_err());
}
