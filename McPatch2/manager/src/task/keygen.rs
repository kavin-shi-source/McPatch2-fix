use rand::RngCore;

use base64ct::{Base64, Encoding};
use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};
use ed25519_dalek::SigningKey;

use crate::web::log::Console;

pub fn task_gen_index_keypair(console: &Console) -> u8 {
    let mut secret = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret);

    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();

    let private_key_der = match signing_key.to_pkcs8_der() {
        Ok(key) => key,
        Err(err) => {
            console.log_error(format!("生成私钥 DER 失败: {}", err));
            return 1;
        }
    };
    let public_key_der = match verifying_key.to_public_key_der() {
        Ok(key) => key,
        Err(err) => {
            console.log_error(format!("生成公钥 DER 失败: {}", err));
            return 1;
        }
    };

    let private_key_base64 = Base64::encode_string(private_key_der.as_bytes());
    let public_key_base64 = Base64::encode_string(public_key_der.as_bytes());

    console.log_info("已生成更新索引签名密钥对");
    console.log_info("请妥善保管私钥，不要提交到仓库");
    console.log_info("");
    console.log_info("[manager config]");
    console.log_info(format!(
        "core.index-signature-private-key = \"{}\"",
        private_key_base64
    ));
    console.log_info("");
    console.log_info("[client config]");
    console.log_info(format!(
        "index-signature-public-key: \"{}\"",
        public_key_base64
    ));

    0
}
