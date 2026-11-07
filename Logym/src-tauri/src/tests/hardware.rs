use super::*;

fn equipo() -> Equipo {
    Equipo { uuid: [0x6d,0x23,0x5b,0x81,0xa2,0x4c,0x47,0x12,0x89,0x65,0x21,0xea,0x73,0x98,0x3c,0x50],
        fabricante: "Synthetic Vendor".into(), modelo: "Synthetic Board X2".into(), serie: "SYNTH-BOARD-4829".into() }
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
