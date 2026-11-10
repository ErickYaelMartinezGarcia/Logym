use sha2::{Digest, Sha256};

#[cfg(test)]
#[path = "tests/hardware.rs"]
mod pruebas;

#[derive(Debug, PartialEq, Eq)]
struct Equipo {
    uuid: [u8; 16],
    fabricante: String,
    modelo: String,
    serie: String,
}

fn normalizar(valor: &str, es_fabricante: bool) -> Result<String, String> {
    if !valor.is_ascii() || valor.bytes().any(|b| b.is_ascii_control() && !b.is_ascii_whitespace()) {
        return Err("SMBIOS contiene caracteres no admitidos".into());
    }
    let texto = valor.split_ascii_whitespace().collect::<Vec<_>>().join(" ").to_ascii_uppercase();
    let simple: String = texto.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    // Solo el fabricante HP admite dos caracteres; producto y serie conservan su mínimo.
    let fabricante_hp = es_fabricante && texto == "HP";
    if (simple.len() < 3 && !fabricante_hp) || simple.bytes().all(|b| b == simple.as_bytes()[0]) ||
        ["TOBEFILLEDBYOEM", "DEFAULTSTRING", "SYSTEMSERIALNUMBER", "BASEBOARDSERIALNUMBER",
         "SERIALNUMBER", "UNKNOWN", "NONE", "NOTAPPLICABLE", "NOTSPECIFIED", "NA", "OEM",
         "SYSTEMPRODUCTNAME", "BASEBOARDPRODUCTNAME", "SYSTEMMANUFACTURER", "BASEBOARDMANUFACTURER",
         "INVALID", "DEFAULT", "123456789", "1234567890", "0123456789"].contains(&simple.as_str()) {
        return Err("SMBIOS contiene identificadores vacíos o genéricos; solicita soporte del fabricante".into());
    }
    Ok(texto)
}

fn identidad(equipo: Equipo) -> Result<String, String> {
    let uuid = equipo.uuid;
    if uuid.iter().all(|b| *b == uuid[0]) ||
        uuid == [3,0,2,0,4,0,5,0,0,6,0,7,0,8,0,9] ||
        uuid == [0,2,0,3,0,4,0,5,0,6,0,7,0,8,0,9] ||
        uuid == [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15] {
        return Err("UUID SMBIOS vacío o genérico; no es posible vincular esta PC".into());
    }
    let fabricante = normalizar(&equipo.fabricante, true)?;
    let modelo = normalizar(&equipo.modelo, false)?;
    let serie = normalizar(&equipo.serie, false)?;
    if serie.chars().filter(|c| c.is_ascii_alphanumeric()).count() < 4 {
        return Err("Número de serie de placa insuficiente".into());
    }
    // Serialización inequívoca y versionada. Nunca incorpora disco, SO, MAC o DPAPI.
    let canonico = serde_json::to_vec(&("LOGYM-HARDWARE-V2", uuid, fabricante, modelo, serie))
        .map_err(|_| "No se pudo normalizar la identidad de hardware")?;
    Ok(format!("LOGYM2-HW-{}", super::hex(&Sha256::digest(canonico))))
}

fn interpretar(raw: &[u8]) -> Result<Equipo, String> {
    let error = || "Tabla SMBIOS incompleta, ambigua o incompatible".to_string();
    // Antes de 2.6 el orden UUID es ambiguo: usar CIM en lugar de adivinarlo.
    if raw.len() < 8 || (raw[1],raw[2]) < (2,6) { return Err("SMBIOS anterior a 2.6: se requiere corroborar el UUID mediante CIM".into()); }
    let longitud = u32::from_le_bytes(raw[4..8].try_into().map_err(|_| error())?) as usize;
    if longitud != raw.len()-8 { return Err(error()); }
    let tabla = &raw[8..];
    let mut offset = 0;
    let mut uuid = None;
    let mut placas = Vec::new();
    let mut terminada = false;
    while offset < tabla.len() {
        let resto = &tabla[offset..];
        if resto.len() < 4 { return Err(error()); }
        let tam = resto[1] as usize;
        if tam < 4 || tam > resto.len() { return Err(error()); }
        let fin = resto[tam..].windows(2).position(|s| s == [0,0]).ok_or_else(error)? + tam;
        let cadenas: Vec<&[u8]> = resto[tam..fin].split(|b| *b == 0).collect();
        let cadena = |indice: u8| -> Result<String,String> {
            if indice == 0 { return Err(error()); }
            let bytes = cadenas.get(indice as usize-1).ok_or_else(error)?;
            Ok(std::str::from_utf8(bytes).map_err(|_| error())?.to_string())
        };
        match resto[0] {
            1 => {
                if tam < 24 || uuid.is_some() { return Err(error()); }
                let mut valor: [u8;16] = resto[8..24].try_into().map_err(|_| error())?;
                // SMBIOS >= 2.6: primeros tres campos little-endian (DMTF DSP0134).
                valor[0..4].reverse(); valor[4..6].reverse(); valor[6..8].reverse();
                uuid = Some(valor);
            },
            2 => {
                // Los campos obligatorios terminan en SerialNumber (offset 7).
                // BoardType, flags y handles son extensiones opcionales en OEM antiguos.
                if tam < 8 { return Err(error()); }
                let principal = (tam > 9 && resto[9] & 1 != 0) || (tam > 13 && resto[13] == 10);
                placas.push((principal, cadena(resto[4])?,cadena(resto[5])?,cadena(resto[7])?));
            },
            127 => { terminada = true; break; },
            _ => (),
        }
        offset += fin+2;
    }
    if !terminada { return Err(error()); }
    let mut principales: Vec<_> = placas.iter().filter(|p|p.0).collect();
    let placa = if principales.len()==1 { principales.remove(0) }
        else if principales.is_empty() && placas.len()==1 { &placas[0] }
        else { return Err(error()); };
    let (_,fabricante,modelo,serie) = placa;
    Ok(Equipo { uuid: uuid.ok_or_else(error)?,fabricante:fabricante.clone(),modelo:modelo.clone(),serie:serie.clone() })
}

#[cfg(windows)]
fn leer() -> Result<Vec<u8>, String> {
    use windows_sys::Win32::System::SystemInformation::GetSystemFirmwareTable;
    // Firma C 'RSMB', no el orden en memoria de una cadena ASCII.
    let proveedor = u32::from_be_bytes(*b"RSMB");
    let tam = unsafe { GetSystemFirmwareTable(proveedor,0,std::ptr::null_mut(),0) };
    if !(8..=16*1024*1024).contains(&tam) {
        return Err("Windows no pudo leer SMBIOS; no se generó identidad de hardware".into());
    }
    let mut datos = vec![0u8;tam as usize];
    let escritos = unsafe { GetSystemFirmwareTable(proveedor,0,datos.as_mut_ptr().cast(),tam) };
    if escritos != tam { return Err("SMBIOS cambió durante su lectura; vuelve a intentar".into()); }
    Ok(datos)
}

fn solicitud_con(mut lectura: impl FnMut() -> Result<Vec<u8>,String>) -> Result<String,String> {
    let primera = identidad(interpretar(&lectura()?)?)?;
    let segunda = identidad(interpretar(&lectura()?)?)?;
    if primera != segunda { return Err("Identificadores de hardware inconsistentes; no se generó solicitud".into()); }
    Ok(primera)
}

#[cfg(windows)]
pub(super) fn solicitud() -> Result<String,String> { resolver(||solicitud_con(leer), leer_cim) }
#[cfg(not(windows))]
pub(super) fn solicitud() -> Result<String,String> { Err("Licencia de hardware disponible solo en Windows".into()) }

// Una fuente inválida/inconsistente no se convierte en un identificador compartido.
fn resolver(principal: impl FnOnce()->Result<String,String>, alternativo: impl FnOnce()->Result<String,String>) -> Result<String,String> {
    match principal() {
        Ok(id) => Ok(id),
        Err(error) if error.contains("inconsistentes") => Err(error),
        Err(error) => alternativo().map_err(|otro|format!("No se obtuvo identidad confiable. SMBIOS: {error}. CIM: {otro}")),
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SistemaCim { uuid: String }
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PlacaCim { fabricante:String, modelo:String, serie:String, principal:bool }
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LecturaCim { sistemas:Vec<SistemaCim>, placas:Vec<PlacaCim> }

fn interpretar_cim(json: &[u8]) -> Result<String,String> {
    let datos: Vec<LecturaCim> = serde_json::from_slice(json).map_err(|_|"Respuesta CIM inválida")?;
    if datos.len()!=2 { return Err("CIM debe proporcionar dos lecturas independientes".into()); }
    let mut ids=Vec::new();
    for mut muestra in datos {
        if muestra.sistemas.len()!=1 { return Err("UUID CIM ausente o ambiguo".into()); }
        let uuid=muestra.sistemas.remove(0).uuid.trim().to_ascii_lowercase();
        if uuid.len()!=36 || ![8,13,18,23].iter().all(|i|uuid.as_bytes()[*i]==b'-') { return Err("UUID CIM malformado".into()); }
        let limpio=uuid.replace('-',"");
        let bytes: [u8;16]=super::decode(&limpio)?.try_into().map_err(|_|"UUID CIM malformado")?;
        let mut principales:Vec<_>=muestra.placas.iter().filter(|p|p.principal).collect();
        let placa=if principales.len()==1 {principales.remove(0)} else if principales.is_empty() && muestra.placas.len()==1 {&muestra.placas[0]} else {return Err("Placa CIM ausente o ambigua".into())};
        ids.push(identidad(Equipo{uuid:bytes,fabricante:placa.fabricante.clone(),modelo:placa.modelo.clone(),serie:placa.serie.clone()})?);
    }
    if ids[0]!=ids[1] { return Err("Identificadores CIM inconsistentes".into()); }
    Ok(ids.remove(0))
}

#[cfg(windows)]
fn leer_cim() -> Result<String,String> {
    use std::{process::{Command,Stdio},os::windows::process::CommandExt,time::{Instant,Duration},io::Read};
    let mut dir=[0u16;32768];
    let len=unsafe{windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW(dir.as_mut_ptr(),dir.len() as u32)} as usize;
    if len==0 || len>=dir.len() {return Err("Windows no proporcionó su directorio del sistema".into());}
    let exe=std::path::PathBuf::from(String::from_utf16(&dir[..len]).map_err(|_|"Ruta Windows inválida")?).join("WindowsPowerShell/v1.0/powershell.exe");
    const SCRIPT:&str=r#"$ErrorActionPreference='Stop'; [Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false); $r=@(1..2 | ForEach-Object { $s=@(Get-CimInstance Win32_ComputerSystemProduct -OperationTimeoutSec 3 | ForEach-Object { @{uuid=[string]$_.UUID} }); $b=@(Get-CimInstance Win32_BaseBoard -OperationTimeoutSec 3 | ForEach-Object { @{fabricante=[string]$_.Manufacturer;modelo=[string]$_.Product;serie=[string]$_.SerialNumber;principal=[bool]$_.HostingBoard} }); @{sistemas=$s;placas=$b} }); ConvertTo-Json -InputObject $r -Depth 5 -Compress"#;
    let mut child=Command::new(exe).args(["-NoLogo","-NoProfile","-NonInteractive","-Command",SCRIPT]).creation_flags(0x08000000).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|_|"No se pudo iniciar la consulta CIM de Windows")?;
    let output=child.stdout.take().ok_or("CIM sin salida")?;
    let reader=std::thread::spawn(move||{let mut bytes=Vec::new();output.take(65537).read_to_end(&mut bytes).map(|_|bytes)});
    let start=Instant::now();
    let status=loop {
        match child.try_wait() { Ok(Some(s))=>break s, Ok(None)=>(), Err(_)=>{let _=child.kill();let _=child.wait();return Err("No se pudo esperar CIM".into());} }
        if start.elapsed()>Duration::from_secs(10) {let _=child.kill();let _=child.wait();let _=reader.join();return Err("La consulta CIM excedió 10 segundos".into());}
        std::thread::sleep(Duration::from_millis(20));
    };
    let bytes=reader.join().map_err(|_|"CIM interrumpido")?.map_err(|_|"No se pudo leer CIM")?;
    if !status.success() || bytes.len()>65536 {return Err("Windows no pudo proporcionar identificadores CIM válidos".into());}
    interpretar_cim(&bytes)
}

#[cfg(windows)]
pub(super) fn diagnostico() -> serde_json::Value {
    let directa=solicitud_con(leer); let cim=leer_cim();
    serde_json::json!({"smbios":directa,"cim":cim})
}
