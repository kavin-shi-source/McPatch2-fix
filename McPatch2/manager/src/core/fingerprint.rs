use std::io::Read;

/// CurseForge 使用的 MurmurHash2 64位变体算法
pub fn calculate_fingerprint(data: &[u8]) -> i64 {
    let seed: u64 = 0x5F4A1C3B;
    let m: u64 = 0xC6A4A7935BD1E995;
    let r: u32 = 47;

    let len = data.len();
    let mut h: u64 = seed ^ (len as u64).wrapping_mul(m);

    let mut chunks = data.chunks_exact(8);
    for chunk in &mut chunks {
        let mut k: u64 = u64::from_le_bytes(chunk.try_into().unwrap());
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

/// 计算文件的 CurseForge 指纹
pub fn fingerprint_file<R: Read>(mut reader: R) -> std::io::Result<i64> {
    let mut buf = Vec::with_capacity(1024 * 1024);
    reader.read_to_end(&mut buf)?;
    Ok(calculate_fingerprint(&buf))
}

/// 计算文件的 SHA-1 哈希
pub fn sha1_hash<R: Read>(mut reader: R) -> std::io::Result<String> {
    use sha2::Digest;
    let mut hasher = sha2::Sha1::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_empty() {
        let fp = calculate_fingerprint(b"");
        assert_ne!(fp, 0);
    }

    #[test]
    fn test_fingerprint_consistency() {
        let data = b"hello world";
        let fp1 = calculate_fingerprint(data);
        let fp2 = calculate_fingerprint(data);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_different_data() {
        let fp1 = calculate_fingerprint(b"hello");
        let fp2 = calculate_fingerprint(b"world");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_sha1_empty() {
        let hash = sha1_hash(std::io::empty()).unwrap();
        assert_eq!(hash, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    }

    #[test]
    fn test_sha1_hello() {
        let hash = sha1_hash("hello world".as_bytes()).unwrap();
        assert_eq!(hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }
}
