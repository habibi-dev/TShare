use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::{
    Aes256Gcm, Key,
    aead::{Aead, KeyInit, OsRng},
};
use base64::{Engine as _, engine::general_purpose};
use sha2::{Digest, Sha256};

const NONCE_SIZE: usize = 12;

fn derive_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

pub fn encrypt(text: &str, password: &str) -> Result<String, &'static str> {
    let key_bytes = derive_key(password);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = GenericArray::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, text.as_bytes())
        .map_err(|_| "encryption failed")?;

    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(output))
}

pub fn decrypt(encoded: &str, password: &str) -> Result<String, &'static str> {
    let data = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| "invalid base64")?;

    if data.len() < NONCE_SIZE + 16 {
        return Err("invalid data length");
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);

    let key_bytes = derive_key(password);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let nonce = GenericArray::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "decryption failed")?;

    String::from_utf8(plaintext).map_err(|_| "invalid utf8")
}
