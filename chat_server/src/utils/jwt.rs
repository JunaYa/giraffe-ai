use jwt_simple::prelude::*;

use crate::{AppError, User};

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISSUER: &str = "chat-server";
const JWT_AUDIENCE: &str = "chat-web";

pub struct EncodingKey(Ed25519KeyPair);
#[allow(unused)]
pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer(JWT_ISSUER).with_audience(JWT_AUDIENCE);
        Ok(self.0.sign(claims)?)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<User, AppError> {
        let ops = VerificationOptions {
            allowed_issuers: Some(HashSet::from_iter([JWT_ISSUER.to_string()])),
            allowed_audiences: Some(HashSet::from_iter([JWT_AUDIENCE.to_string()])),
            ..Default::default()
        };
        let claims = self.0.verify_token(token, Some(ops))?;
        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn jwt_sign_verify_should_work() -> Result<()> {
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");
        let sk = EncodingKey::load(encoding_pem)?;
        let dk = DecodingKey::load(decoding_pem)?;

        let user = User::new(1, "Arjun", "arjun@gmail.com");
        let token = sk.sign(user.clone())?;
        let user2 = dk.verify(&token)?;
        assert_eq!(user, user2);
        Ok(())
    }
}
