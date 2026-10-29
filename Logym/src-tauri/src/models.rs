use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DatosCliente {
    pub nombre: String,
    pub telefono: String,
    #[serde(rename = "telefonoEmergencia")]
    pub telefono_emergencia: String,
    #[serde(rename = "fechaRegistro")]
    pub fecha_registro: String,
    #[serde(rename = "fechaInicioMembresia")]
    pub fecha_inicio_membresia: String,
    #[serde(rename = "fechaFinMembresia")]
    pub fecha_fin_membresia: String,
    #[serde(rename = "fotoBase64")]
    pub foto_base64: String,
    #[serde(rename = "templateHuella")]
    pub template_huella: String,
}

#[derive(Serialize)]
pub struct ClienteTabla {
    pub id: i64,
    pub nombre: String,
    pub telefono: String,
    pub telefono_emergencia: String,
    pub fecha_registro: String,
}

#[derive(Serialize)]
pub struct Metricas {
    pub total_clientes: i64,
    pub asistencias_hoy: i64,
    pub asistencias_ultimos_7_dias: i64,
    pub dia_mas_concurrido: String,
    pub hora_mayor_afluencia: String,
    pub nuevos_semana: i64,
    pub huella_activas: i64,
}

#[derive(Serialize)]
pub struct ClienteMembresia {
    pub id: i64,
    pub nombre: String,
    pub telefono: String,
    pub telefono_emergencia: String,
    pub fecha_registro: String,
    pub fecha_inicio_membresia: String,
    pub fecha_fin_membresia: String,
    pub estado_membresia: String,
    pub dias_restantes: i64,
}

#[derive(Serialize)]
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

#[derive(Deserialize)]
pub struct DatosRenovacion {
    #[serde(rename = "clienteId")]
    pub cliente_id: i64,
}
