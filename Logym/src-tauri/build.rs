fn main() {
    println!("cargo:rerun-if-changed=src/license_public_key.hex");
    if std::env::var("PROFILE").as_deref()==Ok("release") {
        let key = std::fs::read_to_string("src/license_public_key.hex").expect("Falta la clave pública del emisor");
        let key = key.trim();
        assert!(key.len()==64 && key.bytes().all(|b|b.is_ascii_hexdigit()) && key.bytes().any(|b|b!=b'0'),
            "No se permite un release sin clave pública de producción");
    }
    tauri_build::build();

    // OpenSSL vendorizado por SQLCipher puede incluir objetos MSVC que
    // referencian ossl_static.pdb sin distribuir ese archivo. LNK4099
    // solo indica ausencia de símbolos de depuración de esos objetos.
    // Ignorar exclusivamente LNK4099 evita cientos de avisos repetidos,
    // sin suprimir otros warnings ni errores del enlazador.
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-arg=/IGNORE:4099");
}
