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

fn normalizar(valor: &str) -> Result<String, String> {
    if !valor.is_ascii() || valor.bytes().any(|b| b.is_ascii_control() && !b.is_ascii_whitespace()) {
        return Err("SMBIOS contiene caracteres no admitidos".into());
    }
    let texto = valor.split_ascii_whitespace().collect::<Vec<_>>().join(" ").to_ascii_uppercase();
    let simple: String = texto.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if simple.len() < 3 || simple.bytes().all(|b| b == simple.as_bytes()[0]) ||
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
    let fabricante = normalizar(&equipo.fabricante)?;
    let modelo = normalizar(&equipo.modelo)?;
    let serie = normalizar(&equipo.serie)?;
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
    if raw.len() < 8 || (raw[1],raw[2]) < (2,6) { return Err(error()); }
    let longitud = u32::from_le_bytes(raw[4..8].try_into().map_err(|_| error())?) as usize;
    if longitud != raw.len()-8 { return Err(error()); }
    let tabla = &raw[8..];
    let mut offset = 0;
    let mut uuid = None;
    let mut placa = None;
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
                if tam < 25 || uuid.is_some() { return Err(error()); }
                let mut valor: [u8;16] = resto[8..24].try_into().map_err(|_| error())?;
                // SMBIOS >= 2.6: primeros tres campos little-endian (DMTF DSP0134).
                valor[0..4].reverse(); valor[4..6].reverse(); valor[6..8].reverse();
                uuid = Some(valor);
            },
            2 => {
                if tam < 15 || placa.is_some() || resto[13] != 10 { return Err(error()); }
                placa = Some((cadena(resto[4])?,cadena(resto[5])?,cadena(resto[7])?));
            },
            127 => { terminada = true; break; },
            _ => (),
        }
        offset += fin+2;
    }
    if !terminada { return Err(error()); }
    let (fabricante,modelo,serie) = placa.ok_or_else(error)?;
    Ok(Equipo { uuid: uuid.ok_or_else(error)?,fabricante,modelo,serie })
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
    if primera != segunda { return Err("Identificadores de hardware inconsistentes; no se gener? solicitud".into()); }
    Ok(primera)
}

#[cfg(windows)]
pub(super) fn solicitud() -> Result<String,String> { solicitud_con(leer) }
#[cfg(not(windows))]
pub(super) fn solicitud() -> Result<String,String> { Err("Licencia de hardware disponible solo en Windows".into()) }
