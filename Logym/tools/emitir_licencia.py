"""Emisor OFFLINE de licencias Logym. Ejecutar SOLO en la PC del creador.
Requiere: py -m pip install cryptography
La clave privada se guarda FUERA del repositorio. Nunca compartirla.
"""
import argparse
import re
import pathlib
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
from cryptography.hazmat.primitives import serialization

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("accion", choices=["crear-claves", "firmar"])
    parser.add_argument("--solicitud", help="Código LOGYM2-HW-... mostrado por la app")
    parser.add_argument("--privada", required=True, help="Ruta absoluta fuera del repositorio")
    args = parser.parse_args()
    privada = pathlib.Path(args.privada).resolve()
    proyecto = pathlib.Path(__file__).resolve().parents[2]
    if not pathlib.Path(args.privada).is_absolute() or privada == proyecto or proyecto in privada.parents:
        raise SystemExit("La clave privada debe estar fuera del proyecto, en una ruta absoluta.")
    if args.accion == "crear-claves":
        if privada.exists():
            raise SystemExit("Ya existe una clave privada. No se sobrescribe.")
        key = Ed25519PrivateKey.generate()
        privada.write_bytes(key.private_bytes(serialization.Encoding.PEM, serialization.PrivateFormat.PKCS8, serialization.NoEncryption()))
        public = key.public_key().public_bytes(serialization.Encoding.Raw, serialization.PublicFormat.Raw).hex()
        pathlib.Path("license_public_key.hex").write_text(public + "\n", encoding="ascii")
        print("Clave privada creada:", privada.resolve())
        print("Clave pública creada: license_public_key.hex (copiar a src-tauri/src/ antes del build release)")
        print("IMPORTANTE: nunca subas la clave privada a GitHub.")
    else:
        if not args.solicitud or not re.fullmatch(r"LOGYM2-HW-[0-9a-f]{64}", args.solicitud):
            raise SystemExit("Solicitud inválida; se espera LOGYM2-HW- seguido de 64 dígitos hex.")
        key = serialization.load_pem_private_key(privada.read_bytes(), password=None)
        signature = key.sign(args.solicitud.encode("ascii")).hex()
        print("LICENCIA PERMANENTE:\n" + args.solicitud + ":" + signature)

if __name__ == "__main__":
    main()
