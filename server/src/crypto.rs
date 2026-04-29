use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use base64::{Engine, engine::general_purpose::STANDARD};
use anyhow::{anyhow, Result};

pub fn encrypt(master_key: &[u8], plaintext: &str) -> Result<String> {
    let cipher = Aes256Gcm::new_from_slice(master_key)
        .map_err(|e| anyhow!("invalid key: {e}"))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("encryption failed: {e}"))?;

    // store as nonce_b64:ciphertext_b64
    Ok(format!("{}:{}", STANDARD.encode(nonce_bytes), STANDARD.encode(ciphertext)))
}

pub fn decrypt(master_key: &[u8], stored: &str) -> Result<String> {
    let (nonce_b64, ct_b64) = stored
        .split_once(':')
        .ok_or_else(|| anyhow!("invalid encrypted value format"))?;

    let nonce_bytes = STANDARD.decode(nonce_b64)?;
    let ciphertext = STANDARD.decode(ct_b64)?;

    let cipher = Aes256Gcm::new_from_slice(master_key)
        .map_err(|e| anyhow!("invalid key: {e}"))?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| anyhow!("decryption failed: {e}"))?;

    Ok(String::from_utf8(plaintext)?)
}
