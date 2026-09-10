use aes_gcm::{aead::{Aead, AeadCore, KeyInit}, Aes256Gcm, Key, Nonce};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use rand::rngs::OsRng;
use std::error::Error;

pub struct RoomCrypto {
    cipher: Aes256Gcm,
}

impl RoomCrypto {
    pub fn new(room_secret: &str) -> Self {
        let mut key_bytes = [0u8; 32];
        let salt = b"web4_ghost_room_salt_v1";
        pbkdf2_hmac::<Sha256>(
            room_secret.as_bytes(),
            salt,
            600_000,
            &mut key_bytes,
        );
        Self { cipher: Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes)) }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_bytes()).map_err(|e| format!("{:?}", e))?;
        let mut payload = nonce.to_vec();
        payload.extend_from_slice(&ciphertext);
        Ok(payload)
    }

    pub fn decrypt(&self, payload: &[u8]) -> Result<String, Box<dyn Error>> {
        if payload.len() < 12 { return Err("Payload too short".into()); }
        let (nonce_bytes, ciphertext) = payload.split_at(12);
        let plaintext_bytes = self.cipher.decrypt(Nonce::from_slice(nonce_bytes), ciphertext).map_err(|e| format!("{:?}", e))?;
        Ok(String::from_utf8_lossy(&plaintext_bytes).into_owned())
    }
}
