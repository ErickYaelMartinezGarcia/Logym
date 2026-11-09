use super::*;

#[test]
fn mensajes_no_exponen_detalles_y_log_conserva_contexto() {
    for error in [r"SGFPM_OpenDevice: 59; SDK=C:\recursos\sgfplib.dll", "SGFPM_Init: 6", "SQLCipher DPAPI Rust Tauri", "Consulta PnP de Windows: 13"] {
        assert_eq!(publico(error), INICIO);
        let log = diagnostico("inicio", error);
        assert!(log.contains(error));
        for campo in ["pid=", "hilo=", "motivo=", "contexto=inicio", "supervisor=", "generacion_usb=", "sdk_inicializado="] { assert!(log.contains(campo)); }
    }
    assert_eq!(publico("SGFPM_GetImageEx: 55"), CAPTURA);
    assert_eq!(publico("SGFPM_MatchTemplate: 106"), VERIFICACION);
}

#[test]
fn motivos_anidados_se_restauran_y_no_cambian_mensajes_publicos() {
    let inicial = MOTIVO.with(Cell::get);
    {
        let _registro = Tramo::nuevo("registro_capturar");
        assert!(diagnostico("prueba", "SGFPM_OpenDevice: 59").contains("motivo=registro_capturar"));
        {
            let _comparacion = Tramo::nuevo("registro_comparar");
            assert!(diagnostico("prueba", "SDK").contains("motivo=registro_comparar"));
        }
        assert_eq!(MOTIVO.with(Cell::get), "registro_capturar");
        assert_eq!(publico("SGFPM_OpenDevice: 59"), INICIO);
    }
    assert_eq!(MOTIVO.with(Cell::get), inicial);
}
