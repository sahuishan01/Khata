use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};

fn derive_key(secret: &str, user_id_str: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(user_id_str.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

pub fn encrypt_credential(plain: &str, secret: &str, user_id_str: &str) -> Result<String, anyhow::Error> {
    let key = derive_key(secret, user_id_str);
    let cipher = Aes256Gcm::new_from_slice(&key)?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plain.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption error: {:?}", e))?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(combined))
}

pub fn decrypt_credential(encoded: &str, secret: &str, user_id_str: &str) -> Result<String, anyhow::Error> {
    let combined = BASE64.decode(encoded)?;
    if combined.len() < 13 {
        anyhow::bail!("Invalid ciphertext length");
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let key = derive_key(secret, user_id_str);
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext_bytes = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption error: {:?}", e))?;

    let plain = String::from_utf8(plaintext_bytes)?;
    Ok(plain)
}
