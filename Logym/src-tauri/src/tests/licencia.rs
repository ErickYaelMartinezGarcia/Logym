    use super::*;
    use ed25519_dalek::Verifier;
    #[test]
    fn licencia_de_emisor_real_valida_y_rechaza_cambio_instalacion_o_firma() {
        let licencia = include_str!("fixtures/licencia-hardware-sintetica.txt").trim();
        let (id, firma) = licencia.split_once(':').unwrap();
        validar_firma(id,licencia).unwrap();
        assert!(validar_firma(&format!("LOGYM2-HW-{}","0".repeat(64)),licencia).is_err());
        let primer = if firma.starts_with('0') { "1" } else { "0" };
        let alterada = format!("{id}:{primer}{}",&firma[1..]);
        assert!(validar_firma(id,&alterada).is_err());
        assert!(validar_firma(id,"LOGYM2-HW-invalida:00").is_err());
    }
    #[test]
    fn hexadecimal_licencia_rechaza_unicode_impares_y_no_hex() {
        for s in ["0","zz","a€","😃"] {assert!(decode(s).is_err());}
        assert_eq!(decode("aF00").unwrap(),vec![175,0]);
        assert_eq!(hex(&[0,175]),"00af");
    }
    #[test]
    fn clave_publica_configurada_y_firma_falsa_rechazada() {
        let bytes:[u8;32]=decode(CLAVE_PUBLICA.trim()).unwrap().try_into().unwrap();
        let key=VerifyingKey::from_bytes(&bytes).unwrap();
        assert!(key.verify(b"LOGYM2-HW-identidad-sintetica",&Signature::from_bytes(&[0u8;64])).is_err());
    }
