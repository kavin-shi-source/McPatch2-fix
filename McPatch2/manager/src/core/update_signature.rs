use base64ct::{Base64, Encoding};
use ed25519_dalek::pkcs8::DecodePrivateKey;
use ed25519_dalek::Signature;
use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;

use crate::core::data::index_file::VersionIndex;

pub fn canonical_string(index: &VersionIndex) -> String {
    format!(
        "label={}\nfilename={}\noffset={}\nlength={}\nhash={}",
        index.label, index.filename, index.offset, index.len, index.hash
    )
}

pub fn sign_version_index(index: &VersionIndex, private_key_base64: &str) -> Result<String, String> {
    if private_key_base64.trim().is_empty() {
        return Err("missing core.index-signature-private-key".to_owned());
    }

    let key_der = Base64::decode_vec(private_key_base64.trim())
        .map_err(|_| "invalid base64 in core.index-signature-private-key".to_owned())?;
    let signing_key = SigningKey::from_pkcs8_der(&key_der)
        .map_err(|e| format!("invalid pkcs8 private key: {e}"))?;
    let message = canonical_string(index);
    let signature: Signature = signing_key.sign(message.as_bytes());

    Ok(Base64::encode_string(&signature.to_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PRIVATE_KEY_BASE64: &str =
        "MC4CAQAwBQYDK2VwBCIEIA8kef4Ht946ZEJtY6I6noyxVvPBxnfLjWCMORBSwDY6";

    fn sample_index() -> VersionIndex {
        VersionIndex {
            label: "1.0.0".to_owned(),
            filename: "1.0.0.tar".to_owned(),
            offset: 123,
            len: 456,
            hash: "abc123".to_owned(),
            signature: String::new(),
        }
    }

    #[test]
    fn canonical_string_is_stable() {
        let index = sample_index();
        assert_eq!(
            canonical_string(&index),
            "label=1.0.0\nfilename=1.0.0.tar\noffset=123\nlength=456\nhash=abc123"
        );
    }

    #[test]
    fn sign_version_index_returns_signature() {
        let index = sample_index();
        let signature = sign_version_index(&index, TEST_PRIVATE_KEY_BASE64).unwrap();

        assert!(!signature.is_empty());
    }
}
