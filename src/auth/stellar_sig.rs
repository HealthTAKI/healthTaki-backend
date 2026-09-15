use anyhow::{Context, Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

/// Prefix defined by SEP-53 (Stellar signed messages) to namespace signed
/// payloads away from transaction envelopes and prevent cross-protocol
/// replay attacks.
const SIGNED_MESSAGE_PREFIX: &[u8] = b"Stellar Signed Message:\n";

/// Validate that `public_key` is a well-formed Stellar ed25519 account
/// address (a "G..." strkey).
pub fn parse_stellar_public_key(public_key: &str) -> Result<VerifyingKey> {
    let strkey = stellar_strkey::ed25519::PublicKey::from_string(public_key)
        .context("not a valid Stellar public key (expected a G... address)")?;
    VerifyingKey::from_bytes(&strkey.0).context("public key is not a valid ed25519 point")
}

/// Verify a SEP-53 "signed message": the wallet signs
/// `SHA256("Stellar Signed Message:\n" + message)` with its ed25519 key.
/// This is what `@stellar/freighter-api`'s `signMessage` produces, so it
/// lets us authenticate a provider by proving they control the private key
/// for the Stellar address they claim, without ever touching a password or
/// a real transaction.
pub fn verify_signed_message(public_key: &str, message: &str, signature_b64: &str) -> Result<()> {
    let verifying_key = parse_stellar_public_key(public_key)?;

    let signature_bytes = STANDARD
        .decode(signature_b64)
        .context("signature is not valid base64")?;
    let signature_array: [u8; 64] = signature_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("signature must be 64 bytes"))?;
    let signature = Signature::from_bytes(&signature_array);

    let mut hasher = Sha256::new();
    hasher.update(SIGNED_MESSAGE_PREFIX);
    hasher.update(message.as_bytes());
    let digest = hasher.finalize();

    verifying_key
        .verify(&digest, &signature)
        .map_err(|_| anyhow::anyhow!("signature verification failed"))?;

    Ok(())
}

pub fn validate_public_key_format(public_key: &str) -> Result<()> {
    if parse_stellar_public_key(public_key).is_err() {
        bail!("invalid Stellar public key");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn round_trip_self_signed() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let public_key = stellar_strkey::ed25519::PublicKey(signing_key.verifying_key().to_bytes())
            .to_string();

        let message = "hello world";
        let mut hasher = Sha256::new();
        hasher.update(SIGNED_MESSAGE_PREFIX);
        hasher.update(message.as_bytes());
        let digest = hasher.finalize();

        let signature = signing_key.sign(&digest);
        let signature_b64 = STANDARD.encode(signature.to_bytes());

        verify_signed_message(&public_key, message, &signature_b64)
            .expect("self-signed signature should verify");
    }
}
