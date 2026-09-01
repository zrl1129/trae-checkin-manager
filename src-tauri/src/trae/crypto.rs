use aes::cipher::{BlockDecrypt, KeyInit};
use aes::Aes128;
use anyhow::{anyhow, Result};
use base64::Engine;
use sha2::{Digest, Sha512};

const SALT_C: [u8; 64] = [
    191, 192, 216, 250, 122, 246, 220, 97, 31, 254, 98, 27, 8, 72, 71, 176, 135, 99, 96, 18, 127,
    101, 203, 104, 211, 102, 191, 125, 37, 72, 150, 156, 51, 229, 121, 35, 17, 153, 141, 177, 110,
    131, 150, 128, 172, 255, 254, 6, 18, 140, 55, 62, 236, 249, 135, 64, 135, 12, 117, 4, 89, 149,
    168, 209,
];

const SALT_D: [u8; 64] = [
    246, 204, 26, 232, 232, 70, 129, 109, 223, 146, 169, 242, 23, 241, 105, 145, 50, 196, 165,
    42, 254, 120, 3, 54, 244, 207, 209, 85, 53, 6, 138, 106, 175, 148, 31, 204, 186, 186, 165,
    182, 87, 142, 49, 10, 39, 110, 26, 154, 86, 56, 173, 125, 18, 64, 198, 225, 99, 99, 83, 82,
    191, 134, 76, 170,
];

const SALT_A: [u8; 64] = [
    82, 9, 106, 213, 48, 54, 165, 56, 191, 64, 163, 158, 129, 243, 215, 251, 124, 227, 57, 130,
    155, 47, 255, 135, 52, 142, 67, 68, 196, 222, 233, 203, 84, 123, 148, 50, 166, 194, 35, 61,
    238, 76, 149, 11, 66, 250, 195, 78, 8, 46, 161, 102, 40, 217, 36, 178, 118, 91, 162, 73, 109,
    139, 209, 37,
];

const SALT_B: [u8; 64] = [
    31, 221, 168, 51, 136, 7, 199, 49, 177, 18, 16, 89, 39, 128, 236, 95, 96, 81, 127, 169, 25,
    181, 74, 13, 45, 229, 122, 159, 147, 201, 156, 239, 160, 224, 59, 77, 174, 42, 245, 176, 200,
    235, 187, 60, 131, 83, 153, 97, 23, 43, 4, 126, 186, 119, 214, 38, 225, 105, 20, 99, 85, 33,
    12, 125,
];

fn xor_salts(a: &[u8; 64], b: &[u8; 64]) -> [u8; 64] {
    let mut r = [0u8; 64];
    for i in 0..64 {
        r[i] = a[i] ^ b[i];
    }
    r
}

enum EncType {
    Aes,
    AesPrivate,
}

fn detect_enc_type(header: &[u8]) -> Option<EncType> {
    if header.len() < 6 {
        return None;
    }
    if header[0] == 0x74 && header[1] == 0x63 && header[2] == 0x05 && header[3] == 0x10
        && header[4] == 0x00 && header[5] == 0x00
    {
        Some(EncType::Aes)
    } else if header[0] == 18 && header[1] == 57 && header[2] == 32 && header[3] == 32
        && header[4] == 2 && header[5] == 3
    {
        Some(EncType::AesPrivate)
    } else {
        None
    }
}

fn derive_key_iv(random_bytes: &[u8], enc_type: &EncType) -> ([u8; 16], [u8; 16]) {
    let salt = match enc_type {
        EncType::AesPrivate => xor_salts(&SALT_C, &SALT_D),
        EncType::Aes => xor_salts(&SALT_A, &SALT_B),
    };

    let mut hasher = Sha512::new();
    hasher.update(random_bytes);
    let hash_of_random = hasher.finalize();

    let mut combined = [0u8; 128];
    combined[..64].copy_from_slice(&hash_of_random);
    combined[64..].copy_from_slice(&salt);

    let final_hash = Sha512::digest(&combined);

    let mut aes_key = [0u8; 16];
    aes_key.copy_from_slice(&final_hash[0..16]);
    let mut iv = [0u8; 16];
    iv.copy_from_slice(&final_hash[16..32]);

    (aes_key, iv)
}

fn aes_cbc_decrypt(key: &[u8; 16], iv: &[u8; 16], ciphertext: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err(anyhow!("ciphertext length invalid"));
    }
    let cipher = Aes128::new_from_slice(key).map_err(|e| anyhow!("AES key error: {}", e))?;
    let mut result = Vec::with_capacity(ciphertext.len());
    let mut prev = *iv;
    for chunk in ciphertext.chunks(16) {
        let mut block = aes::cipher::generic_array::GenericArray::clone_from_slice(chunk);
        cipher.decrypt_block(&mut block);
        for i in 0..16 {
            block[i] ^= prev[i];
        }
        result.extend_from_slice(&block);
        prev.copy_from_slice(chunk);
    }
    Ok(result)
}

fn remove_pkcs7(data: &[u8]) -> Result<&[u8]> {
    if data.is_empty() {
        return Err(anyhow!("empty data for pkcs7"));
    }
    let pad_len = *data.last().unwrap() as usize;
    if pad_len == 0 || pad_len > 16 || pad_len > data.len() {
        return Err(anyhow!("invalid pkcs7 padding"));
    }
    Ok(&data[..data.len() - pad_len])
}

pub fn decrypt_storage_value(base64_value: &str) -> Result<String> {
    let raw = base64::engine::general_purpose::STANDARD.decode(base64_value.trim())?;
    if raw.len() < 39 {
        return Err(anyhow!("decoded data too short"));
    }
    let header = &raw[0..6];
    let random_bytes = &raw[6..38];
    let ciphertext = &raw[38..];
    let enc_type = detect_enc_type(header).ok_or_else(|| anyhow!("unknown encryption type"))?;
    let (key, iv) = derive_key_iv(random_bytes, &enc_type);
    let decrypted = aes_cbc_decrypt(&key, &iv, ciphertext)?;
    let decrypted = remove_pkcs7(&decrypted)?;
    if decrypted.len() < 64 {
        return Err(anyhow!("decrypted data too short for hash"));
    }
    let stored_hash = &decrypted[0..64];
    let plaintext = &decrypted[64..];
    let computed_hash = Sha512::digest(plaintext);
    if stored_hash != computed_hash.as_slice() {
        return Err(anyhow!("hash verification failed"));
    }
    Ok(String::from_utf8_lossy(plaintext).to_string())
}
