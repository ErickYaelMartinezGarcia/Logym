use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct DatosCliente {
    pub nombre: String,
    pub telefono: String,
    #[serde(default, rename = "telefonoEmergencia")]
    pub telefono_emergencia: String,
    #[serde(default, rename = "fechaRegistro")]
    pub fecha_registro: String,
    #[serde(default, rename = "fechaInicioMembresia")]
    pub fecha_inicio_membresia: String,
    #[serde(default, rename = "fechaFinMembresia")]
    pub fecha_fin_membresia: String,
    #[serde(default, rename = "fotoBase64")]
    pub foto_base64: String,
    #[serde(default, rename = "templateHuella")]
    pub template_huella: String,
}

#[derive(Debug, Deserialize)]
pub struct DatosRenovacion {
    #[serde(rename = "clienteId", alias = "cliente_id")]
    pub cliente_id: i64,
}

#[derive(Debug, Serialize)]
pub struct ClienteTabla {
    pub id: i64,
    pub nombre: String,
    pub telefono: String,
    pub fecha_fin_membresia: String,
    pub foto_base64: String,
}

#[derive(Debug, Serialize)]
pub struct ClienteMembresia {
    pub id: i64,
    pub nombre: String,
    pub telefono: String,
    pub estado: String,
    pub dias_restantes: i64,
    pub fecha_fin_membresia: String,
}

#[derive(Debug, Serialize)]
pub struct ClienteCheckIn {
    pub id: i64,
    pub nombre: String,
    pub telefono: String,
    pub telefono_emergencia: String,
    pub foto_base64: String,
    pub fecha_registro: String,
    pub fecha_inicio_membresia: String,
    pub fecha_fin_membresia: String,
    pub estado_membresia: String,
    pub dias_restantes: i64,
}

#[derive(Debug, Serialize)]
pub struct Metricas {
    pub total_clientes: i64,
    pub asistencias_hoy: i64,
    pub nuevos_semana: i64,
    pub huella_activas: i64,
    pub asistencias_ultimos_7_dias: i64,
    pub dia_mas_concurrido: String,
    pub hora_mayor_afluencia: String,
}
/// Datos editables; no permite modificar huellas ni identificadores.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualizacionCliente {
    pub nombre: String,
    pub telefono: String,
    pub telefono_emergencia: String,
    pub fecha_inicio_membresia: String,
    pub fecha_fin_membresia: String,
}
