use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use std::fmt;

#[derive(Debug)]
pub struct IdentityError(String);
impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IdentityError: {}", self.0)
    }
}

pub struct SovereignIdentity {
    pub keypair: SigningKey,
    pub did: String,
}

impl SovereignIdentity {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        let keypair = SigningKey::generate(&mut csprng);
        let public_key = VerifyingKey::from(&keypair);
        let mut hasher = Sha256::new();
        hasher.update(public_key.to_bytes());
        Self {
            keypair,
            did: format!("did:web4:{}", hex::encode(hasher.finalize())),
        }
    }
}
