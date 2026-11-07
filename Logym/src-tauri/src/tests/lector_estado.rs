    use super::*;

    #[test]
    fn reinsertion_invalidates_old_manager_even_for_same_device() {
        let mut usb = UsbTracker::default();
        let ids = vec!["USB\\VID_1162&PID_2200\\reader".into()];
        assert!(usb.update(Ok(ids.clone())));
        let old = usb.generation;
        assert!(usb.ready(old));
        assert!(!usb.update(Ok(ids.clone())));
        usb.update(Ok(vec![]));
        assert!(!usb.ready(old));
        usb.update(Ok(ids));
        assert!(!usb.ready(old));
        assert!(usb.ready(usb.generation));
    }

    #[test]
    fn pnp_failure_does_not_confirm_readiness() {
        let mut usb = UsbTracker::default();
        usb.update(Err("PnP error".into()));
        assert!(!usb.ready(usb.generation));
        assert!(es_secugen("USB\\VID_1162&PID_2200\\123"));
        assert!(es_secugen("USB\\VID_1162&PID_2201\\123"));
        assert!(!es_secugen("USB\\VID_1162&PID_22000\\123"));
        assert!(!es_secugen("USB\\VID_11620&PID_2200\\123"));
    }

    #[test]
    fn initialized_sdk_without_capture_keeps_alert_and_stale_proof_is_rejected() {
        let mut usb = UsbTracker::default();
        usb.update(Ok(vec!["reader".into()]));
        let old = usb.generation;
        let mut state = EstadoLector { estado: Estado::Inicializando, detalle: String::new(), revision: 0,
            usb_presente: false, sdk_inicializado: false, captura_comprobada: false };
        assert!(aplicar_evidencia(&mut state, &usb, old, false));
        assert_eq!(state.estado, Estado::Recuperando);
        assert!(state.sdk_inicializado);
        assert!(!state.captura_comprobada);
        assert!(aplicar_evidencia(&mut state, &usb, old, true));
        assert_eq!(state.estado, Estado::Disponible);
        usb.update(Ok(vec![]));
        aplicar_usb(&mut state, &usb);
        assert_eq!(state.estado, Estado::NoDisponible);
        assert!(!state.captura_comprobada);
        usb.update(Ok(vec!["reader".into()]));
        aplicar_usb(&mut state, &usb);
        assert_eq!(state.estado, Estado::Recuperando);
        assert!(state.usb_presente);
        assert!(!state.sdk_inicializado);
        assert!(!state.captura_comprobada);
        assert!(!aplicar_evidencia(&mut state, &usb, old, true));
        assert!(aplicar_evidencia(&mut state, &usb, usb.generation, false));
        assert_eq!(state.estado, Estado::Recuperando);
    }

pub(crate) fn simular_usb(ids: Vec<String>) -> u64 {
    let mut usb = USB.lock().unwrap_or_else(|e| e.into_inner());
    usb.update(Ok(ids));
    usb.generation
}
