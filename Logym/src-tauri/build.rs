fn main() {
    tauri_build::build();

    // OpenSSL vendorizado por SQLCipher puede incluir objetos MSVC que
    // referencian ossl_static.pdb sin distribuir ese archivo. LNK4099
    // solo indica ausencia de símbolos de depuración de esos objetos.
    // Ignorar exclusivamente LNK4099 evita cientos de avisos repetidos,
    // sin suprimir otros warnings ni errores del enlazador.
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-arg=/IGNORE:4099");
}
