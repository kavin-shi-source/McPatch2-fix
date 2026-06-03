use std::io::Read;
use std::path::Path;

use sha2::Digest;

/// 计算一个文件的指纹（sha256）
pub fn calculate_fingerprint(patch: &Path) -> String {
    let file = match std::fs::File::open(patch) {
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

/// CurseForge 使用的 MurmurHash2 64 位变体算法
pub fn calculate_mod_fingerprint(data: &[u8]) -> i64 {
    let seed: u64 = 0x5F4A1C3B;
    let m: u64 = 0xC6A4A7935BD1E995;
    let r: u32 = 47;

    let len = data.len();
    let mut h: u64 = seed ^ (len as u64).wrapping_mul(m);

    let mut chunks = data.chunks_exact(8);
    for chunk in &mut chunks {
        let mut k = u64::from_le_bytes(chunk.try_into().unwrap());
        k = k.wrapping_mul(m);
        k ^= k >> r;
        k = k.wrapping_mul(m);
        h ^= k;
        h = h.wrapping_mul(m);
    }

    let remaining = chunks.remainder();
    if !remaining.is_empty() {
        let mut k: u64 = 0;
        for (i, &byte) in remaining.iter().enumerate() {
            k ^= (byte as u64) << (i * 8);
        }
        k = k.wrapping_mul(m);
        k ^= k >> r;
        k = k.wrapping_mul(m);
        h ^= k;
    }

    h ^= h >> r;
    h = h.wrapping_mul(m);
    h ^= h >> r;

    h as i64
}

pub fn fingerprint_file<R: Read>(mut reader: R) -> std::io::Result<i64> {
    let mut buf = Vec::with_capacity(1024 * 1024);
    reader.read_to_end(&mut buf)?;
    Ok(calculate_mod_fingerprint(&buf))
}

pub fn sha1_hash<R: Read>(mut reader: R) -> std::io::Result<String> {
    use sha1::Digest;

    let mut hasher = sha1::Sha1::new();
    let mut buf = [0u8; 65536];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
