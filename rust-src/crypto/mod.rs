use std::io;
use sha1::Digest;

/// AES-256 decryption for EGG archives
pub fn aes_decrypt(data: &[u8], password: &str) -> io::Result<Vec<u8>> {
    use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyIvInit};

    // EGG uses PBKDF2 to derive key from password
    let salt = &data[..16]; // First 16 bytes are salt
    let encrypted = &data[16..];

    // Derive key using PBKDF2-HMAC-SHA1
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];

    pbkdf2::pbkdf2_hmac::<sha1::Sha1>(
        password.as_bytes(),
        salt,
        1000,
        &mut key,
    );

    // IV is derived from key
    iv.copy_from_slice(&key[..16]);

    // Decrypt using AES-256-CBC
    type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

    if encrypted.len() % 16 != 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid AES encrypted data length"));
    }

    let mut decrypted = encrypted.to_vec();
    let cipher = Aes256CbcDec::new_from_slices(&key, &iv)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("AES init error: {e}")))?;

    cipher.decrypt_padded_mut::<NoPadding>(&mut decrypted)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("AES decrypt error: {e}")))?;

    // Remove PKCS7 padding
    if let Some(&pad_len) = decrypted.last() {
        if pad_len > 0 && pad_len <= 16 && decrypted.len() >= pad_len as usize {
            let valid = decrypted.len() - pad_len as usize;
            decrypted.truncate(valid);
        }
    }

    Ok(decrypted)
}

/// Zip-compatible decryption (PKZIP traditional encryption)
pub fn zip_decrypt(data: &[u8], password: &str) -> Vec<u8> {
    // PKZIP encryption uses a simple 3-key system
    let mut k0: u32 = 0x12345678;
    let mut k1: u32 = 0x23456789;
    let mut k2: u32 = 0x34567890;

    // Initialize keys with password
    for &b in password.as_bytes() {
        update_keys(b, &mut k0, &mut k1, &mut k2);
    }

    // Decrypt data
    let mut output = Vec::with_capacity(data.len());
    for &b in data {
        let temp = (k2 & 0xFFFF) | 2;
        let decrypted = b ^ (((temp * (temp ^ 1)) >> 8) as u8);
        update_keys(decrypted, &mut k0, &mut k1, &mut k2);
        output.push(decrypted);
    }

    // Remove 12-byte header (encryption verification)
    if output.len() > 12 {
        output[12..].to_vec()
    } else {
        output
    }
}

fn update_keys(byte: u8, k0: &mut u32, k1: &mut u32, k2: &mut u32) {
    *k0 = crc32_update(*k0, byte);
    *k1 = (*k1).wrapping_add(*k0 & 0xFF);
    *k1 = (*k1).wrapping_mul(134775813).wrapping_add(1);
    *k2 = crc32_update(*k2, ((*k1 >> 24) & 0xFF) as u8);
}

fn crc32_update(crc: u32, byte: u8) -> u32 {
    // Simple CRC32 update for PKZIP encryption
    let table: [u32; 256] = generate_crc32_table();
    (crc >> 8) ^ table[((crc & 0xFF) ^ byte as u32) as usize]
}

fn generate_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for i in 0..256 {
        let mut crc = i as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
        table[i] = crc;
    }
    table
}
