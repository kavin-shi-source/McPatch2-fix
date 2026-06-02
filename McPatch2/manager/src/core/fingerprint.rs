use std::io::Read;
use std::path::Path;

use sha2::Digest;

/// 计算一个文件的指纹（sha256）
pub fn calculate_fingerprint(patch: &Path) -> String {
    let mut file = match std::fs::File::open(patch) {
        Ok(f) => f,
        Err(e) => return format!("文件读取失败: {:?}", e),
    };

    let mut hasher = sha2::Sha256::new();
    let mut reader = std::io::BufReader::new(file);
    let mut buffer = [0u8; 8192];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                hasher.update(&buffer[..n]);
            }
            Err(e) => return format!("文件读取失败: {:?}", e),
        }
    }

    let hash = hasher.finalize();

    base16ct::lower::encode_string(&hash)
}
