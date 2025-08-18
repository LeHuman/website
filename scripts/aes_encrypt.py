#!/usr/bin/env python3
"""
aes_encryptor.py

Utility for generating AES keys and encrypting text or files.
Designed to work with client-side decryption using WebCrypto API.

Usage (CLI):
    # Generate a random 256-bit key
    python aes_encryptor.py --gen-key

    # Encrypt a text string with a key
    python aes_encryptor.py --key <hexkey> --text "Secret message"

    # Encrypt a file with a key
    python aes_encryptor.py --key <hexkey> --file input.png
"""

from __future__ import annotations
import argparse
import json
import os
from typing import Tuple
from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from cryptography.hazmat.primitives import padding
from cryptography.hazmat.backends import default_backend


def generate_key() -> bytes:
    """Generate a 256-bit AES key."""
    return os.urandom(32)


def encrypt_bytes(data: bytes, key: bytes) -> Tuple[bytes, bytes]:
    """
    Encrypt raw bytes with AES-256-CBC.

    Args:
        data: Plaintext bytes.
        key: 32-byte AES key.

    Returns:
        Tuple of (iv, ciphertext).
    """
    iv = os.urandom(16)
    padder = padding.PKCS7(128).padder()
    padded = padder.update(data) + padder.finalize()

    cipher = Cipher(algorithms.AES(key), modes.CBC(iv), backend=default_backend())
    encryptor = cipher.encryptor()
    ciphertext = encryptor.update(padded) + encryptor.finalize()

    return iv, ciphertext


def decrypt_bytes(key: bytes, iv: bytes, ct: bytes) -> bytes:
    """
    Decrypt raw bytes with AES-256-CBC and PKCS7 padding.

    Args:
        key: 32-byte AES key.
        iv: Initialization vector.
        ct: Ciphertext bytes.

    Returns:
        Decrypted plaintext bytes.
    """

    cipher = Cipher(algorithms.AES(key), modes.CBC(iv), backend=default_backend())
    decryptor = cipher.decryptor()
    padded_text = decryptor.update(ct) + decryptor.finalize()

    # Unpad the plaintext using PKCS7
    unpadder = padding.PKCS7(128).unpadder()
    text = unpadder.update(padded_text) + unpadder.finalize()

    return text


def encrypt_text(text: str, key: bytes) -> Tuple[bytes, bytes]:
    """Encrypt a UTF-8 text string with the given key."""
    return encrypt_bytes(text.encode("utf-8"), key)


def encrypt_file(path: str, key: bytes) -> Tuple[bytes, bytes]:
    """Encrypt the contents of a file with the given key."""
    with open(path, "rb") as f:
        data = f.read()
    return encrypt_bytes(data, key)


def bytes_to_hex(data: bytes) -> str:
    """Convert bytes to a lowercase hex string."""
    return data.hex()


def main() -> None:
    parser = argparse.ArgumentParser(description="AES-256-CBC encryption utility for Zola/JS integration.")
    parser.add_argument("--gen-key", action="store_true", help="Generate a random 256-bit key (hex).")
    parser.add_argument("--key", type=str, help="Hex-encoded AES key (256-bit).")
    parser.add_argument("--key-file", type=str, help="Hex-encoded AES key (256-bit) as a file.")
    parser.add_argument("--text", type=str, help="Text to encrypt.")
    parser.add_argument("--file", type=str, help="File to encrypt.")
    parser.add_argument("--iv", type=str, help="IV to use for decryption.")
    parser.add_argument("--ct", type=str, help="Cipher text to use for decryption.")
    parser.add_argument("-o", "--output", type=str, help="Set an output file.")

    args = parser.parse_args()

    if args.gen_key:
        key = generate_key()
        print(f"Generated key (hex): {bytes_to_hex(key)}")
        return

    if not args.key and args.key_file:
        with open(args.key_file, encoding="utf-8") as f:
            args.key = f.readline()

    if not args.key:
        parser.error("You must provide --key <hexkey> for encryption unless using --gen-key.")

    try:
        key = bytes.fromhex(args.key)
    except ValueError as exc:
        raise ValueError("Key must be valid hex.") from exc

    if len(key) != 32:
        raise ValueError("Key must be 32 bytes (256-bit) in hex (64 hex chars).")

    if args.iv and args.ct:
        try:
            iv = bytes.fromhex(args.iv)
            ct = bytes.fromhex(args.ct)
        except ValueError as exc:
            raise ValueError("Key must be valid hex.") from exc

        d_bytes = decrypt_bytes(key, iv, ct)

        print("Decoded bytes:")
        print(d_bytes.decode())
    elif args.text:
        iv, ct = encrypt_text(args.text, key)
        iv = bytes_to_hex(iv)
        ct = bytes_to_hex(ct)

        if args.output:
            with open(args.output, mode="w", encoding="UTF-8") as f:
                json.dump({"iv": iv, "ct": ct}, f, indent=4)
                f.write("\n")
        else:
            print(f"IV (hex): {iv}")
            print(f"Ciphertext (hex): {ct}")
    elif args.file:
        iv, ct = encrypt_file(args.file, key)
        iv = bytes_to_hex(iv)
        ct = bytes_to_hex(ct)

        if args.output:
            with open(args.output, mode="w", encoding="UTF-8") as f:
                json.dump({"iv": iv, "ct": ct}, f, indent=4)
                f.write("\n")
        else:
            print(f"IV (hex): {iv}")
            print(f"Ciphertext (hex): {ct}")
    else:
        parser.error("Must supply either --text or --file for encryption.")


if __name__ == "__main__":
    main()
